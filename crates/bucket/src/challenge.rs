use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    fs::{create_dir, read_dir, read_to_string, write, File},
    io::AsyncRead,
};
use tracing::debug;

use crate::traits::{init_dir, BucketError};

#[derive(Debug)]
pub struct ChallengeBucket {
    pub name: String,
    pub path: PathBuf,
    pub locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreRule {
    pub initial: i32,
    pub minimum: i32,
    pub decay: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    name: String,
    primary: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagList(pub Vec<Tag>);

#[derive(Serialize, Deserialize)]
pub struct ChallengeConfig {
    pub name: String,
    pub hidden: bool,
    pub tag: TagList,
    pub score_rule: ScoreRule,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeImage {
    pub tag: String,
    pub cpu: f64,
    pub mem: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeEnv {
    pub port: u16,
    pub images: Vec<ChallengeImage>,
}

impl ChallengeBucket {
    pub async fn open(
        root_path: impl AsRef<Path>, name: impl AsRef<str>, locked: bool,
    ) -> Result<Self, BucketError> {
        let challenge_path = root_path.as_ref().join(name.as_ref());
        if !challenge_path.exists() {
            return Err(BucketError::PathDoesNotExist(
                challenge_path.display().to_string(),
            ));
        }
        Ok(Self {
            name: name.as_ref().to_owned(),
            path: challenge_path,
            locked,
        })
    }

    pub async fn new(
        root_path: impl AsRef<Path>, name: impl AsRef<str>, config: ChallengeConfig,
    ) -> Result<Self, BucketError> {
        let challenge_path = root_path.as_ref().join(name.as_ref());
        if challenge_path.exists() {
            return Err(BucketError::PathConflict(
                challenge_path.display().to_string(),
            ));
        }
        create_dir(&challenge_path).await?;
        init_dir!(challenge_path, "images");
        init_dir!(challenge_path, "mapped");
        init_dir!(challenge_path, "checker");
        init_dir!(challenge_path, "src");
        init_dir!(challenge_path, "static");
        write(
            &challenge_path.join("config.toml"),
            toml::to_string_pretty(&config)?,
        )
        .await?;

        Ok(Self {
            name: name.as_ref().to_owned(),
            path: challenge_path,
            locked: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn set_config(&self, config: Value) -> Result<(), BucketError> {
        if !self.locked {
            return Err(BucketError::NeedLocking);
        }
        let config: ChallengeConfig = serde_json::from_value(config)?;
        write(
            &self.path.join("config.toml"),
            toml::to_string_pretty(&config)?,
        )
        .await?;
        Ok(())
    }

    pub async fn config(&self) -> Result<ChallengeConfig, BucketError> {
        let config = toml::from_str(&read_to_string(&self.path.join("config.toml")).await?)?;
        Ok(config)
    }

    pub async fn set_env(&self, config: Value) -> Result<(), BucketError> {
        if !self.locked {
            return Err(BucketError::NeedLocking);
        }
        let config: ChallengeEnv = serde_json::from_value(config)?;
        write(
            &self.path.join("env.toml"),
            toml::to_string_pretty(&config)?,
        )
        .await?;

        Ok(())
    }

    pub async fn env(&self) -> Result<Option<ChallengeEnv>, BucketError> {
        let path = self.path.join("env.toml");
        if !path.exists() {
            return Ok(None);
        }
        let config = toml::from_str(&read_to_string(&path).await?)?;
        Ok(Some(config))
    }

    async fn upload_file(
        &self, dest: impl AsRef<str>, name: impl AsRef<str>,
        mut stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        if !self.locked {
            return Err(BucketError::NeedLocking);
        }
        if !matches!(
            dest.as_ref(),
            "images" | "mapped" | "scripts" | "src" | "static"
        ) {
            return Err(BucketError::PathDoesNotExist(dest.as_ref().to_owned()));
        }
        let dest_path = self.path.join(dest.as_ref()).join(name.as_ref());
        let mut file = tokio::fs::File::create(&dest_path).await?;
        tokio::io::copy(&mut stdin, &mut file).await?;

        Ok(())
    }

    pub async fn upload_static(
        &self, name: impl AsRef<str>, stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        self.upload_file("static", name, stdin).await
    }

    pub async fn upload_images(
        &self, name: impl AsRef<str>, stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        self.upload_file("images", name, stdin).await
    }

    pub async fn upload_mapped(
        &self, name: impl AsRef<str>, stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        self.upload_file("mapped", name, stdin).await
    }

    pub async fn upload_scripts(
        &self, name: impl AsRef<str>, stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        self.upload_file("scripts", name, stdin).await
    }

    pub async fn upload_src(
        &self, name: impl AsRef<str>, stdin: impl AsyncRead + Send + Unpin,
    ) -> Result<(), BucketError> {
        self.upload_file("src", name, stdin).await
    }

    pub async fn get_static_files(&self) -> Result<Vec<String>, BucketError> {
        let mut files = vec![];
        let mut dir = read_dir(&self.path.join("static")).await?;
        while let Some(entry) = dir.next_entry().await? {
            let entry_file = entry.file_name().to_string_lossy().to_string();
            if entry_file.starts_with('.') {
                continue;
            }
            files.push(entry_file);
        }
        Ok(files)
    }

    pub async fn get_mapped_files(&self) -> Result<Vec<String>, BucketError> {
        let mut files = vec![];
        let mut dir = read_dir(&self.path.join("mapped")).await?;
        while let Some(entry) = dir.next_entry().await? {
            let entry_file = entry.file_name().to_string_lossy().to_string();
            if entry_file.starts_with('.') {
                continue;
            }
            files.push(entry_file);
        }
        Ok(files)
    }

    pub async fn get_mapped_file(&self, requested_id: i64) -> Result<Option<String>, BucketError> {
        let files = self.get_mapped_files().await?;
        if files.is_empty() {
            return Ok(None);
        }
        let file_index = requested_id as usize % files.len();
        Ok(Some(files[file_index].clone()))
    }

    pub async fn download_file(&self, path: impl AsRef<Path>) -> Result<File, BucketError> {
        debug!("downloading file at path: {:?}", path.as_ref());
        Ok(File::open(path).await?)
    }

    fn ensure_prefix(
        &self, sub_folder: impl AsRef<str>, file: impl AsRef<str>,
    ) -> Result<PathBuf, BucketError> {
        let sub_folder = self.path.join(sub_folder.as_ref()).canonicalize()?;
        let file_path = self.path.join(file.as_ref()).canonicalize()?;
        if !file_path.starts_with(sub_folder) {
            Err(BucketError::PathTraversal)
        } else {
            Ok(file_path)
        }
    }

    pub async fn download_static(&self, name: impl AsRef<str>) -> Result<File, BucketError> {
        debug!("downloading static file {}", name.as_ref());
        self.download_file(&self.ensure_prefix("static", format!("static/{}", name.as_ref()))?)
            .await
    }

    pub async fn download_mapped(&self, name: impl AsRef<str>) -> Result<File, BucketError> {
        debug!("downloading mapped file {}", name.as_ref());
        self.download_file(&self.ensure_prefix("mapped", format!("mapped/{}", name.as_ref()))?)
            .await
    }

    pub fn hash(&self) -> String {
        let mut hasher = ring::digest::Context::new(&ring::digest::SHA256);
        hasher.update(self.path.to_string_lossy().as_bytes());
        hasher
            .finish()
            .as_ref()
            .iter()
            .fold(String::new(), |mut acc, b| {
                acc.push_str(&format!("{:02x}", b));
                acc
            })
        // .map(|b| format!("{:02x}", b))
        // .collect::<String>()
    }
}
