use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow, bail};
use chrono::{DateTime, Utc, serde::ts_seconds};
use r2s_config::bucket;
use r2s_database::game_registry_source;
use serde::{Deserialize, Serialize};
use tokio::{
  fs::{create_dir_all, read_to_string, remove_dir_all, write},
  process::Command,
};
use tracing::{info, warn};

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

pub struct PublishRegistryRelease<'a> {
  pub game_key: &'a str,
  pub release_id: &'a str,
  pub manifest_body: &'a str,
  pub instance_id: &'a str,
  pub base_url: &'a str,
  pub sync_token: &'a str,
  pub published_at: DateTime<Utc>,
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

pub async fn publish_release_to_registry(
  bucket_config: &Option<bucket::Config>, source: &game_registry_source::Model,
  release: PublishRegistryRelease<'_>,
) -> anyhow::Result<()> {
  if !source.publish_enabled {
    bail!("registry source {} is not publish-enabled", source.name);
  }

  let cache_dir = fetch_registry_source(bucket_config, source).await?;
  let release_dir = cache_dir
    .join("games")
    .join(release.game_key)
    .join("releases");
  let upstream_dir = cache_dir
    .join("games")
    .join(release.game_key)
    .join("upstreams")
    .join(release.instance_id);
  create_dir_all(&release_dir).await?;
  create_dir_all(&upstream_dir).await?;

  let release_path = release_dir.join(format!("{}.toml", release.release_id));
  if release_path.exists() {
    let existing = read_to_string(&release_path).await?;
    if existing != release.manifest_body {
      bail!(
        "release file {} already exists with different content",
        release_path.display()
      );
    }
  } else {
    write(&release_path, release.manifest_body).await?;
  }

  let upstream_body = r2s_config::toml::to_string_pretty(&UpstreamAdvertisement {
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
  })?;
  let upstream_path =
    upstream_dir.join(format!("{}.toml", release.published_at.timestamp_millis()));
  write(&upstream_path, upstream_body).await?;

  let status = git_status_porcelain(&cache_dir).await?;
  if status.trim().is_empty() {
    info!(source=%source.name, game_key=%release.game_key, release_id=%release.release_id, "registry source already up to date");
    return Ok(());
  }

  run_git(Some(&cache_dir), &["add".to_owned(), "--all".to_owned()]).await?;
  run_git_with_env(
    Some(&cache_dir),
    &[
      "commit".to_owned(),
      "--author".to_owned(),
      "platform <platform@private.ret.sh.cn>".to_owned(),
      "-m".to_owned(),
      format!(
        ":sparkles: publish release {}@{}",
        release.game_key,
        short_release_id(release.release_id)
      ),
    ],
    &[
      ("GIT_COMMITTER_NAME", "platform"),
      ("GIT_COMMITTER_EMAIL", "platform@private.ret.sh.cn"),
    ],
  )
  .await?;
  run_git(
    Some(&cache_dir),
    &[
      "push".to_owned(),
      "origin".to_owned(),
      format!("HEAD:{}", source.branch),
    ],
  )
  .await?;
  Ok(())
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

async fn git_status_porcelain(path: &Path) -> anyhow::Result<String> {
  run_git(
    Some(path),
    &[
      "status".to_owned(),
      "--porcelain".to_owned(),
      "-u".to_owned(),
    ],
  )
  .await
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
