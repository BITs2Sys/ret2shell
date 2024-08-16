use r2s_config::cluster::RegistryConfig;
use serde::{Deserialize, Serialize};
use tempdir::TempDir;
use tokio::{io::AsyncRead, process::Command};
use tracing::debug;

use crate::ClusterError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registry {
  credentials: Option<RegistryConfig>,
}

#[derive(Deserialize)]
struct Repository {
  repositories: Vec<String>,
}

#[derive(Deserialize)]
struct Tags {
  tags: Vec<String>,
}

impl Registry {
  pub fn new(c: RegistryConfig) -> Self {
    Self {
      credentials: Some(c),
    }
  }

  fn base(&self) -> Result<String, ClusterError> {
    let credentials = self
      .credentials
      .as_ref()
      .ok_or(ClusterError::ConfigNeeded)?;
    if let Some(ref username) = credentials.username {
      if let Some(ref password) = credentials.password {
        Ok(format!(
          "{}:{}@{}",
          username,
          password,
          credentials.server.clone()
        ))
      } else {
        Err(ClusterError::MissingField("password".to_string()))
      }
    } else {
      Ok(credentials.server.clone())
    }
  }

  fn api_base(&self) -> Result<String, ClusterError> {
    let credentials = self
      .credentials
      .as_ref()
      .ok_or(ClusterError::ConfigNeeded)?;
    Ok(format!(
      "{}://{}/v2",
      if credentials.insecure {
        "http"
      } else {
        "https"
      },
      credentials.server.clone()
    ))
  }

  pub async fn repositories(&self) -> Result<Vec<String>, ClusterError> {
    let api_base = self.api_base()?;
    let mut result: Vec<String> = Vec::new();
    let mut last = String::new();
    loop {
      let res = match last {
        ref s if s.is_empty() => reqwest::get(&format!("{}/_catalog?n=1000", api_base)).await?,
        ref s => reqwest::get(&format!("{}/_catalog?n=1000&last={}", api_base, s)).await?,
      };
      let body: Repository = res.json().await?;
      let repositories = body.repositories;
      if repositories.is_empty() {
        break;
      }
      last = repositories.last().unwrap().clone();
      result.extend(repositories);
    }
    Ok(result)
  }

  pub async fn images(&self, repository: &str) -> Result<Vec<String>, ClusterError> {
    let api_base = self.api_base()?;
    let res = reqwest::get(&format!("{api_base}/{repository}/tags/list")).await?;
    let body: Tags = res.json().await?;
    Ok(body.tags)
  }

  pub async fn upload_image(
    &self, name: &str, mut stdin: impl AsyncRead + Send + Unpin,
  ) -> Result<(), ClusterError> {
    let tmp_dir = TempDir::new("ret2shell")?;
    let file_path = tmp_dir.path().join(name);
    let mut file = tokio::fs::File::create(&file_path).await?;
    debug!("uploading file to path: {:?}", file_path);
    tokio::io::copy(&mut stdin, &mut file).await?;
    // get tag name without file extension
    let repo = name.split('.').next().unwrap();
    let mut args = vec![
      "copy".to_string(),
      format!("docker-archive:{}", name),
      format!("docker://{}/{}:latest", self.base()?, repo),
    ];
    if self.credentials.clone().is_some_and(|c| c.insecure) {
      args.push("--dest-tls-verify=false".to_string());
    }
    let output = Command::new("skopeo")
      .current_dir(&tmp_dir)
      // .arg("copy")
      // .arg(format!("docker-archive:{}", name))
      // .arg(format!("docker://{}/{}:latest", self.base()?, repo))
      .args(&args)
      .output()
      .await?;
    if output.status.success() {
      Ok(())
    } else {
      Err(ClusterError::UploadFailed(
        String::from_utf8_lossy(&output.stderr).to_string(),
      ))
    }
  }
}
