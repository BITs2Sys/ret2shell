use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use chrono::Utc;
use r2s_config::bucket;
use r2s_database::{challenge, game, game_registry_source};
use r2s_migrator::Database;
use rand::Rng;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use tokio::fs::{create_dir_all, read_to_string, write};
use tracing::info;

const SYNC_DIR: &str = ".sync";
const SOURCES_DIR: &str = "sources";
const MIRRORS_DIR: &str = "mirrors";
const JOBS_DIR: &str = "jobs";
const MEDIA_PART_DIR: &str = "media-part";
const INSTANCE_ID_FILE: &str = "instance-id";
const DEFAULT_SOURCE_NAME: &str = "ret2shell-official";
const DEFAULT_SOURCE_URL: &str = "https://github.com/ret2shell/game-registry";
const DEFAULT_SOURCE_BRANCH: &str = "main";

pub mod manifest;
pub mod registry;

pub async fn initialize(db: &Database, config: &Option<bucket::Config>) -> anyhow::Result<()> {
  let root = sync_root(config)?;
  create_dir_all(root.join(SOURCES_DIR)).await?;
  create_dir_all(root.join(MIRRORS_DIR)).await?;
  create_dir_all(root.join(JOBS_DIR)).await?;
  create_dir_all(root.join(MEDIA_PART_DIR)).await?;

  let instance_id = ensure_instance_id(&root).await?;
  backfill_game_sync_fields(db).await?;
  backfill_challenge_display_order(db).await?;
  ensure_default_registry_source(db).await?;

  info!(instance_id=%instance_id, path=%root.display(), "sync workspace initialized");
  Ok(())
}

pub fn sync_root(config: &Option<bucket::Config>) -> anyhow::Result<PathBuf> {
  let bucket_path = config
    .as_ref()
    .map(|config| PathBuf::from(&config.path))
    .ok_or_else(|| anyhow!("bucket configuration not found"))?;
  Ok(bucket_path.join(SYNC_DIR))
}

async fn ensure_instance_id(root: &Path) -> anyhow::Result<String> {
  let path = root.join(INSTANCE_ID_FILE);
  if let Ok(existing) = read_to_string(&path).await {
    let existing = existing.trim().to_owned();
    if !existing.is_empty() {
      return Ok(existing);
    }
  }

  let instance_id = generate_instance_id();
  write(&path, format!("{instance_id}\n")).await?;
  Ok(instance_id)
}

pub async fn instance_id(config: &Option<bucket::Config>) -> anyhow::Result<String> {
  let root = sync_root(config)?;
  ensure_instance_id(&root).await
}

pub fn source_cache_dir(
  config: &Option<bucket::Config>, source_id: i64,
) -> anyhow::Result<PathBuf> {
  Ok(
    sync_root(config)?
      .join(SOURCES_DIR)
      .join(source_id.to_string()),
  )
}

pub fn job_workspace_dir(config: &Option<bucket::Config>, job_id: &str) -> anyhow::Result<PathBuf> {
  Ok(sync_root(config)?.join(JOBS_DIR).join(job_id))
}

fn generate_instance_id() -> String {
  let mut bytes = [0_u8; 16];
  rand::rng().fill_bytes(&mut bytes);
  bytes[6] = (bytes[6] & 0x0F) | 0x40;
  bytes[8] = (bytes[8] & 0x3F) | 0x80;

  format!(
    "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
    bytes[0],
    bytes[1],
    bytes[2],
    bytes[3],
    bytes[4],
    bytes[5],
    bytes[6],
    bytes[7],
    bytes[8],
    bytes[9],
    bytes[10],
    bytes[11],
    bytes[12],
    bytes[13],
    bytes[14],
    bytes[15]
  )
}

async fn backfill_game_sync_fields(db: &Database) -> anyhow::Result<()> {
  let games = game::Entity::find()
    .filter(game::Column::HostType.eq(game::HostType::Game))
    .all(&db.conn)
    .await?;

  for current in games {
    let next_sync_key = current.sync_key.clone().or_else(|| current.bucket.clone());
    let next_sync_token = current
      .sync_token
      .clone()
      .or_else(|| Some(nanoid::nanoid!()));

    if next_sync_key == current.sync_key && next_sync_token == current.sync_token {
      continue;
    }

    let model = game::ActiveModel {
      id: ActiveValue::Unchanged(current.id),
      sync_key: ActiveValue::Set(next_sync_key),
      sync_token: ActiveValue::Set(next_sync_token),
      ..Default::default()
    };
    model.update(&db.conn).await.with_context(|| {
      format!(
        "failed to backfill game sync fields for game {}",
        current.id
      )
    })?;
  }

  Ok(())
}

async fn backfill_challenge_display_order(db: &Database) -> anyhow::Result<()> {
  let challenges = challenge::Entity::find()
    .order_by_asc(challenge::Column::GameId)
    .order_by_asc(challenge::Column::Id)
    .all(&db.conn)
    .await?;

  let mut previous_game_id = None;
  let mut display_order = 0_i32;
  for current in challenges {
    if previous_game_id != Some(current.game_id) {
      previous_game_id = Some(current.game_id);
      display_order = 1;
    } else {
      display_order += 1;
    }

    if current.display_order == display_order {
      continue;
    }

    let model = challenge::ActiveModel {
      id: ActiveValue::Unchanged(current.id),
      display_order: ActiveValue::Set(display_order),
      ..Default::default()
    };
    model.update(&db.conn).await.with_context(|| {
      format!(
        "failed to backfill challenge display order for challenge {}",
        current.id
      )
    })?;
  }

  Ok(())
}

async fn ensure_default_registry_source(db: &Database) -> anyhow::Result<()> {
  let existing = game_registry_source::get_list(&db.conn).await?;
  if existing.iter().any(|source| {
    source.name == DEFAULT_SOURCE_NAME
      || (source.git_url == DEFAULT_SOURCE_URL && source.branch == DEFAULT_SOURCE_BRANCH)
  }) {
    return Ok(());
  }

  game_registry_source::create(
    &db.conn,
    game_registry_source::Model {
      id: 0,
      name: DEFAULT_SOURCE_NAME.to_owned(),
      git_url: DEFAULT_SOURCE_URL.to_owned(),
      branch: DEFAULT_SOURCE_BRANCH.to_owned(),
      enabled: true,
      priority: 0,
      publish_enabled: false,
      private_source: false,
      last_fetched_at: None,
      last_error: None,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    },
  )
  .await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::generate_instance_id;

  #[test]
  fn generated_instance_id_looks_like_uuid_v4() {
    let id = generate_instance_id();
    assert_eq!(id.len(), 36);
    assert_eq!(&id[8..9], "-");
    assert_eq!(&id[13..14], "-");
    assert_eq!(&id[18..19], "-");
    assert_eq!(&id[23..24], "-");
    assert_eq!(&id[14..15], "4");
    assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"));
  }
}
