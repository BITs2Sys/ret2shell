use std::collections::BTreeSet;

use anyhow::{Context, bail};
use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use once_cell::sync::Lazy;
use r2s_bucket::game::{GameBucket, GameDocument};
use r2s_captcha::sha256sum_str;
use r2s_config::cluster::ChallengeEnv;
use r2s_database::{challenge, game};
use regex::Regex;
use serde::Serialize;

pub const RELEASE_REF_PREFIX: &str = "refs/ret2shell/releases";

static MEDIA_HASH_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"/media\?hash=([0-9a-f]{64})"#).expect("valid media hash regex"));
static HASH_ONLY_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"^[0-9a-f]{64}$"#).expect("valid plain hash regex"));

#[derive(Serialize)]
pub struct BuiltReleaseManifest {
  pub release_id: String,
  pub snapshot_commit: String,
  pub manifest_sha256: String,
  pub manifest_body: String,
}

#[derive(Serialize)]
pub struct ReleaseManifest {
  pub spec_version: i32,
  pub kind: String,
  pub game_key: String,
  pub release_id: String,
  pub snapshot_commit: String,
  #[serde(with = "ts_seconds")]
  pub published_at: DateTime<Utc>,
  pub first_party_instance_id: String,
  pub game: GameManifest,
  pub challenges: Vec<ChallengeManifest>,
  pub assets: ManifestAssets,
}

#[derive(Serialize)]
pub struct GameManifest {
  pub name: String,
  pub brief: String,
  pub host_type: String,
  #[serde(with = "ts_seconds")]
  pub start_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub end_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub register_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub archive_at: DateTime<Utc>,
  pub team_size: i32,
  pub weight: i32,
  pub sync_policy: i32,
  pub can_register_after_started: bool,
  pub cover_kind: Option<String>,
  pub cover_value: Option<String>,
  pub logo_kind: Option<String>,
  pub logo_value: Option<String>,
  pub show_answer_after_archive: bool,
  pub show_hints_after_archive: bool,
}

#[derive(Serialize)]
pub struct ChallengeManifest {
  pub key: String,
  pub order: i32,
  pub hidden: bool,
  pub score: i32,
  #[serde(with = "ts_seconds_option", default = "Option::default")]
  pub release_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option", default = "Option::default")]
  pub archive_at: Option<DateTime<Utc>>,
}

#[derive(Default, Serialize)]
pub struct ManifestAssets {
  pub media_hashes: Vec<String>,
}

pub fn release_ref(release_id: &str) -> String {
  format!("{RELEASE_REF_PREFIX}/{release_id}")
}

pub async fn build_release_manifest(
  game: &game::Model, game_bucket: &GameBucket, challenges: &[challenge::Model], game_key: &str,
  first_party_instance_id: &str, published_at: DateTime<Utc>,
) -> anyhow::Result<BuiltReleaseManifest> {
  let snapshot_commit = game_bucket.git.get_head().await?;
  let release_id = snapshot_commit.clone();
  let assets = build_assets(game, game_bucket, challenges).await?;
  let manifest = ReleaseManifest {
    spec_version: 1,
    kind: "release".to_owned(),
    game_key: game_key.to_owned(),
    release_id: release_id.clone(),
    snapshot_commit: snapshot_commit.clone(),
    published_at,
    first_party_instance_id: first_party_instance_id.to_owned(),
    game: GameManifest {
      name: game.name.clone(),
      brief: game.brief.clone(),
      host_type: match game.host_type {
        game::HostType::Training => "training".to_owned(),
        game::HostType::Game => "game".to_owned(),
      },
      start_at: game.start_at,
      end_at: game.end_at,
      register_at: game.register_at,
      archive_at: game.archive_at,
      team_size: game.team_size,
      weight: game.weight,
      sync_policy: game.access_policy.sync,
      can_register_after_started: game.can_register_after_started,
      cover_kind: asset_kind(game.cover.as_deref()),
      cover_value: game.cover.clone(),
      logo_kind: asset_kind(game.logo.as_deref()),
      logo_value: game.logo.clone(),
      show_answer_after_archive: game.archive_policy.challenge.show_answer,
      show_hints_after_archive: game.archive_policy.challenge.show_hints,
    },
    challenges: challenges
      .iter()
      .map(|challenge| {
        Ok(ChallengeManifest {
          key: challenge
            .bucket
            .clone()
            .ok_or_else(|| anyhow::anyhow!("challenge {} does not have a bucket", challenge.id))?,
          order: challenge.display_order,
          hidden: challenge.hidden,
          score: challenge.score,
          release_at: challenge.release_at,
          archive_at: challenge.archive_at,
        })
      })
      .collect::<anyhow::Result<Vec<_>>>()?,
    assets,
  };
  let manifest_body = r2s_config::toml::to_string_pretty(&manifest)?;
  Ok(BuiltReleaseManifest {
    release_id,
    snapshot_commit,
    manifest_sha256: sha256sum_str(&manifest_body),
    manifest_body,
  })
}

async fn build_assets(
  game: &game::Model, game_bucket: &GameBucket, challenges: &[challenge::Model],
) -> anyhow::Result<ManifestAssets> {
  let mut media_hashes = BTreeSet::new();
  collect_hash_from_value(&mut media_hashes, game.cover.as_deref());
  collect_hash_from_value(&mut media_hashes, game.logo.as_deref());

  for doc in [
    GameDocument::Readme,
    GameDocument::Rules,
    GameDocument::Training,
  ] {
    if let Some(content) = game_bucket.read_document(doc).await? {
      collect_hashes_from_text(&mut media_hashes, &content);
    }
  }

  for challenge in challenges {
    let challenge_bucket = game_bucket
      .at(
        challenge
          .bucket
          .as_ref()
          .context("challenge bucket missing while building release manifest")?,
      )
      .await?;
    collect_hashes_from_text(&mut media_hashes, &challenge_bucket.description().await?);
    collect_hashes_from_text(&mut media_hashes, &challenge_bucket.answer().await?);
    for hint in challenge_bucket.hints().await?.hints {
      collect_hashes_from_text(&mut media_hashes, &hint.content);
    }
    if let Some(env) = challenge_bucket.env().await? {
      ensure_publishable_env(&env, challenge.id)?;
    }
  }

  Ok(ManifestAssets {
    media_hashes: media_hashes.into_iter().collect(),
  })
}

fn ensure_publishable_env(env: &ChallengeEnv, challenge_id: i64) -> anyhow::Result<()> {
  if env.images.iter().any(|image| image.internal_managed) {
    bail!(
      "challenge {challenge_id} uses internal-managed images; Phase 6 registry asset publication is not implemented yet"
    );
  }
  Ok(())
}

fn collect_hash_from_value(media_hashes: &mut BTreeSet<String>, value: Option<&str>) {
  let Some(value) = value else {
    return;
  };
  if HASH_ONLY_REGEX.is_match(value) {
    media_hashes.insert(value.to_owned());
  }
}

fn collect_hashes_from_text(media_hashes: &mut BTreeSet<String>, content: &str) {
  for capture in MEDIA_HASH_REGEX.captures_iter(content) {
    if let Some(hash) = capture.get(1) {
      media_hashes.insert(hash.as_str().to_owned());
    }
  }
}

fn asset_kind(value: Option<&str>) -> Option<String> {
  let value = value?;
  if HASH_ONLY_REGEX.is_match(value) {
    Some("media_hash".to_owned())
  } else if value.contains("://") {
    Some("external_url".to_owned())
  } else {
    Some("repo_path".to_owned())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;

  use super::{collect_hash_from_value, collect_hashes_from_text};

  #[test]
  fn collect_hashes_from_markdown_media_paths() {
    let mut hashes = BTreeSet::new();
    collect_hashes_from_text(
      &mut hashes,
      "![img](/media?hash=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef)",
    );
    assert_eq!(hashes.len(), 1);
  }

  #[test]
  fn collect_hash_from_direct_media_hash_value() {
    let mut hashes = BTreeSet::new();
    collect_hash_from_value(
      &mut hashes,
      Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
    );
    assert_eq!(hashes.len(), 1);
  }
}
