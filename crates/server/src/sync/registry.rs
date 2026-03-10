use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, Utc, serde::ts_seconds};
use r2s_config::bucket;
use r2s_database::game_registry_source;
use serde::{Deserialize, Serialize};
use tokio::{
  fs::{create_dir_all, read_to_string, remove_dir_all},
  process::Command,
};
use tracing::warn;

use super::source_cache_dir;

const REGISTRY_KIND: &str = "ret2shell-game-registry";

#[derive(Deserialize)]
struct RegistryMetadata {
  spec_version: i32,
  kind: String,
}

#[derive(Serialize)]
struct UpstreamAdvertisement {
  spec_version: i32,
  kind: String,
  status: String,
  game_key: String,
  release_id: String,
  instance_id: String,
  role: String,
  #[serde(with = "ts_seconds")]
  published_at: DateTime<Utc>,
  base_url: String,
  auth_mode: String,
  sync_token: String,
  protocol_version: i32,
}

pub struct RegistryPublicationRequest<'a> {
  pub game_key: &'a str,
  pub release_id: &'a str,
  pub manifest_body: &'a str,
  pub instance_id: &'a str,
  pub base_url: &'a str,
  pub sync_token: &'a str,
  pub published_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ManualRegistryPublication {
  pub registry_source_name: String,
  pub registry_git_url: String,
  pub registry_branch: String,
  pub release_file_path: String,
  pub release_file_content: String,
  pub upstream_file_path: String,
  pub upstream_file_content: String,
  pub suggested_pr_title: String,
}

pub async fn fetch_registry_source(
  bucket_config: &Option<bucket::Config>, source: &game_registry_source::Model,
) -> anyhow::Result<PathBuf> {
  let cache_dir = source_cache_dir(bucket_config, source.id)?;
  let mut freshly_cloned = false;
  if cache_dir.exists() && !cache_dir.join(".git").exists() {
    remove_dir_all(&cache_dir).await.ok();
  }

  if !cache_dir.exists() {
    freshly_cloned = true;
    if let Some(parent) = cache_dir.parent() {
      create_dir_all(parent).await?;
    }
    run_git(
      None,
      &[
        "clone".to_owned(),
        "--branch".to_owned(),
        source.branch.clone(),
        "--single-branch".to_owned(),
        source.git_url.clone(),
        cache_dir.to_string_lossy().to_string(),
      ],
    )
    .await?;
  } else {
    run_git(
      Some(&cache_dir),
      &[
        "remote".to_owned(),
        "set-url".to_owned(),
        "origin".to_owned(),
        source.git_url.clone(),
      ],
    )
    .await?;
    run_git(
      Some(&cache_dir),
      &[
        "fetch".to_owned(),
        "origin".to_owned(),
        source.branch.clone(),
        "--prune".to_owned(),
      ],
    )
    .await?;
    run_git(
      Some(&cache_dir),
      &[
        "checkout".to_owned(),
        "-B".to_owned(),
        source.branch.clone(),
        format!("origin/{}", source.branch),
      ],
    )
    .await?;
  }

  if let Err(err) = validate_registry_checkout(&cache_dir).await {
    if freshly_cloned {
      remove_dir_all(&cache_dir).await.ok();
    }
    return Err(err);
  }
  Ok(cache_dir)
}

pub async fn remove_registry_source_cache(
  bucket_config: &Option<bucket::Config>, source_id: i64,
) -> anyhow::Result<()> {
  let cache_dir = source_cache_dir(bucket_config, source_id)?;
  if cache_dir.exists() {
    remove_dir_all(cache_dir).await?;
  }
  Ok(())
}

pub fn build_manual_registry_publication(
  source: &game_registry_source::Model, release: RegistryPublicationRequest<'_>,
) -> anyhow::Result<ManualRegistryPublication> {
  if !source.publish_enabled {
    bail!("registry source {} is not publish-enabled", source.name);
  }

  let upstream_body = build_upstream_advertisement_body(&release)?;
  Ok(ManualRegistryPublication {
    registry_source_name: source.name.clone(),
    registry_git_url: source.git_url.clone(),
    registry_branch: source.branch.clone(),
    release_file_path: format!(
      "games/{}/releases/{}.toml",
      release.game_key, release.release_id
    ),
    release_file_content: release.manifest_body.to_owned(),
    upstream_file_path: format!(
      "games/{}/upstreams/{}/{}.toml",
      release.game_key,
      release.instance_id,
      release.published_at.timestamp_millis()
    ),
    upstream_file_content: upstream_body,
    suggested_pr_title: format!(
      ":sparkles: publish release {}@{}",
      release.game_key,
      short_release_id(release.release_id)
    ),
  })
}

fn build_upstream_advertisement_body(
  release: &RegistryPublicationRequest<'_>,
) -> anyhow::Result<String> {
  Ok(r2s_config::toml::to_string_pretty(
    &UpstreamAdvertisement {
      spec_version: 1,
      kind: "upstream".to_owned(),
      status: "active".to_owned(),
      game_key: release.game_key.to_owned(),
      release_id: release.release_id.to_owned(),
      instance_id: release.instance_id.to_owned(),
      role: "first_party".to_owned(),
      published_at: release.published_at,
      base_url: release.base_url.to_owned(),
      auth_mode: "sync_token".to_owned(),
      sync_token: release.sync_token.to_owned(),
      protocol_version: 1,
    },
  )?)
}

async fn validate_registry_checkout(path: &Path) -> anyhow::Result<()> {
  let metadata_path = path.join("registry.toml");
  let body = read_to_string(&metadata_path)
    .await
    .with_context(|| format!("registry metadata not found at {}", metadata_path.display()))?;
  validate_registry_metadata_body(&body)
    .with_context(|| format!("invalid registry metadata at {}", metadata_path.display()))
}

fn validate_registry_metadata_body(body: &str) -> anyhow::Result<()> {
  let metadata: RegistryMetadata = r2s_config::toml::from_str(body)?;
  if metadata.spec_version != 1 {
    bail!(
      "unsupported registry spec version {}",
      metadata.spec_version
    );
  }
  if metadata.kind != REGISTRY_KIND {
    bail!("invalid registry kind {}", metadata.kind);
  }
  Ok(())
}

async fn run_git(current_dir: Option<&Path>, args: &[String]) -> anyhow::Result<String> {
  run_git_with_env(current_dir, args, &[]).await
}

async fn run_git_with_env(
  current_dir: Option<&Path>, args: &[String], envs: &[(&str, &str)],
) -> anyhow::Result<String> {
  let mut cmd = Command::new("git");
  if let Some(current_dir) = current_dir {
    cmd.current_dir(current_dir);
  }
  for (key, value) in envs {
    cmd.env(key, value);
  }
  cmd.args(args);
  let output = cmd.output().await?;
  if output.status.success() {
    Ok(String::from_utf8(output.stdout)?)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    warn!(
      ?args,
      ?stderr,
      "git command failed while handling registry source"
    );
    Err(anyhow!(stderr))
  }
}

fn short_release_id(release_id: &str) -> &str {
  let short_len = release_id.len().min(12);
  &release_id[..short_len]
}

#[cfg(test)]
mod tests {
  use super::{short_release_id, validate_registry_metadata_body};

  #[test]
  fn short_release_id_keeps_short_values() {
    assert_eq!(short_release_id("abc"), "abc");
    assert_eq!(short_release_id("1234567890abcdef"), "1234567890ab");
  }

  #[test]
  fn validate_registry_metadata_body_accepts_v1_registry() {
    validate_registry_metadata_body(
      r#"
spec_version = 1
kind = "ret2shell-game-registry"
"#,
    )
    .expect("valid registry metadata should pass");
  }

  #[test]
  fn validate_registry_metadata_body_rejects_invalid_kind() {
    let err = validate_registry_metadata_body(
      r#"
spec_version = 1
kind = "something-else"
"#,
    )
    .expect_err("invalid registry kind should fail");

    assert!(format!("{err:#}").contains("invalid registry kind"));
  }

  #[test]
  fn validate_registry_metadata_body_rejects_unsupported_version() {
    let err = validate_registry_metadata_body(
      r#"
spec_version = 2
kind = "ret2shell-game-registry"
"#,
    )
    .expect_err("unsupported registry spec version should fail");

    assert!(format!("{err:#}").contains("unsupported registry spec version"));
  }
}
