//! The bucket system, which is used to store the files and serve `wsvc` service.
//!
//!

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::{create_dir_all, read_dir},
    io,
};
use wsvc::{
    fs::RepoGuard,
    model::{ObjectId, Record, Repository},
};

use crate::entity::challenge;

#[derive(Error, Debug)]
pub enum BucketError {
    #[error("challenge must have a bucket: {0:?}")]
    LackOfBucketId(challenge::Model),
    #[error("bucket init conflict: {0}")]
    BucketInitConflict(String),
    #[error("bucket not exist: {0}")]
    BucketNotExist(String),
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("wsvc general error: {0}")]
    WsvcError(#[from] wsvc::WsvcError),
    #[error("wsvc fs error: {0}")]
    WsvcFsError(#[from] wsvc::fs::WsvcFsError),
    #[error("repository is empty: {0}")]
    RepositoryIsEmpty(String),
}

pub struct Bucket {
    storage_root_dir: PathBuf,
    pub challenge: challenge::Model,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntryItem {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: Vec<FileEntryItem>,
}

impl Bucket {
    fn new(storage_root_dir: impl AsRef<Path>, challenge: &challenge::Model) -> Self {
        Self {
            storage_root_dir: storage_root_dir.as_ref().to_owned(),
            challenge: challenge.clone(),
        }
    }

    pub async fn initialize(
        storage_root_dir: impl AsRef<Path>,
        challenge: &challenge::Model,
    ) -> Result<Self, BucketError> {
        let bucket_dir = storage_root_dir.as_ref().join(
            challenge
                .bucket
                .as_ref()
                .ok_or(BucketError::LackOfBucketId(challenge.clone()))?,
        );
        if bucket_dir.exists() {
            return Err(BucketError::BucketInitConflict(
                bucket_dir.to_string_lossy().to_string(),
            ));
        }
        let repo_dir = bucket_dir.join("repo");
        create_dir_all(&repo_dir).await?;
        let _ = Repository::new(repo_dir, false).await?;
        create_dir_all(bucket_dir.join("workflows")).await?;
        create_dir_all(bucket_dir.join("static")).await?;

        Ok(Self::new(storage_root_dir, challenge))
    }

    async fn repo_dir(&self) -> Result<PathBuf, BucketError> {
        let bucket_dir = self
            .storage_root_dir
            .join(
                self.challenge
                    .bucket
                    .as_ref()
                    .ok_or(BucketError::LackOfBucketId(self.challenge.clone()))?,
            )
            .join("repo");
        if !bucket_dir.exists() {
            return Err(BucketError::BucketNotExist(
                bucket_dir.to_string_lossy().to_string(),
            ));
        }
        Ok(bucket_dir)
    }

    pub async fn repo(&self) -> Result<Repository, BucketError> {
        let repo_dir = self.repo_dir().await?;
        Ok(Repository::open(repo_dir, false).await?)
    }

    #[allow(dead_code)]
    pub async fn workflows_dir(&self) -> Result<PathBuf, BucketError> {
        let bucket_dir = self
            .storage_root_dir
            .join(
                self.challenge
                    .bucket
                    .as_ref()
                    .ok_or(BucketError::LackOfBucketId(self.challenge.clone()))?,
            )
            .join("workflows");
        if !bucket_dir.exists() {
            return Err(BucketError::BucketInitConflict(
                bucket_dir.to_string_lossy().to_string(),
            ));
        }
        Ok(bucket_dir)
    }

    #[allow(dead_code)]
    pub async fn static_dir(&self) -> Result<PathBuf, BucketError> {
        let bucket_dir = self
            .storage_root_dir
            .join(
                self.challenge
                    .bucket
                    .as_ref()
                    .ok_or(BucketError::LackOfBucketId(self.challenge.clone()))?,
            )
            .join("static");
        if !bucket_dir.exists() {
            return Err(BucketError::BucketInitConflict(
                bucket_dir.to_string_lossy().to_string(),
            ));
        }
        Ok(bucket_dir)
    }

    #[allow(dead_code)]
    pub async fn checkout_latest(&self) -> Result<(), BucketError> {
        let repo = self.repo().await?;
        let guard = RepoGuard::new(&repo).await?;
        repo.checkout_record(
            &repo
                .get_latest_record()
                .await?
                .ok_or(BucketError::RepositoryIsEmpty(
                    self.challenge.bucket.clone().unwrap_or_default(),
                ))?
                .hash,
            &self.repo_dir().await?,
        )
        .await?;
        drop(guard);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn checkout_record(&self, hash: &ObjectId) -> Result<(), BucketError> {
        let repo = self.repo().await?;
        let guard = RepoGuard::new(&repo).await?;
        repo.checkout_record(hash, &self.repo_dir().await?).await?;
        drop(guard);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn commit_record(
        &self,
        author: impl AsRef<str>,
        message: impl AsRef<str>,
    ) -> Result<Record, BucketError> {
        let repo = self.repo().await?;
        let guard = RepoGuard::new(&repo).await?;
        let result = repo
            .commit_record(&self.repo_dir().await?, author, message)
            .await?;
        drop(guard);
        Ok(result)
    }

    #[allow(dead_code)]
    pub async fn get_records(&self) -> Result<Vec<Record>, BucketError> {
        let repo = self.repo().await?;
        Ok(repo.get_records().await?)
    }

    #[allow(dead_code)]
    pub async fn get_repo_tree(&self) -> Result<FileEntryItem, BucketError> {
        let repo_dir = self.repo_dir().await?;
        scan_tree(&repo_dir).await
    }
}

#[async_recursion::async_recursion(?Send)]
async fn scan_tree(path: &Path) -> Result<FileEntryItem, BucketError> {
    if path.is_file() {
        Ok(FileEntryItem {
            name: path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new(""))
                .to_string_lossy()
                .to_string(),
            is_dir: false,
            size: path.metadata()?.len(),
            children: Vec::new(),
        })
    } else {
        let mut item = FileEntryItem {
            name: path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new(""))
                .to_string_lossy()
                .to_string(),
            is_dir: true,
            size: 4 * 1024,
            children: Vec::new(),
        };
        let mut entries = read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            item.children.push(scan_tree(&entry.path()).await?);
        }
        Ok(item)
    }
}
