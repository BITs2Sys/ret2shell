use std::{collections::BTreeMap, path::Path};

use axum::{
  Extension, Json,
  body::{Body, to_bytes},
  extract::State,
  http::{HeaderValue, Method, Request, Uri, header::AUTHORIZATION},
  response::IntoResponse,
};
use chrono::{DateTime, Utc};
use deunicode::deunicode_with_tofu;
use futures::TryStreamExt;
use heck::ToSnakeCase;
use r2s_bucket::{Bucket, git::Git};
use r2s_captcha::sha256sum_str;
use r2s_database::{challenge, game, game_release, game_remote_sync, hint, media};
use r2s_media::Media;
use r2s_migrator::Database;
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tokio::{
  fs::{create_dir_all, read_dir, remove_dir_all, rename},
  process::Command,
};
use tokio_util::io::StreamReader;

use crate::{
  middleware::auth::Token,
  sync::{self, manifest},
  traits::{GlobalState, HTTPClient, ResponseError},
};

#[derive(Deserialize)]
pub(super) struct DirectDiscoverRequest {
  pub base_url: String,
  pub sync_token: Option<String>,
  pub game_key: Option<String>,
  pub release_id: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct DirectImportRequest {
  pub base_url: String,
  pub sync_token: Option<String>,
  pub game_key: String,
  pub release_id: String,
}

#[derive(Serialize)]
pub(super) struct DirectImportResponse {
  pub game_id: i64,
  pub game_key: String,
  pub release_id: String,
  pub bucket: String,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RemoteSyncInfo {
  pub instance_id: String,
  pub base_url: String,
  pub protocol_version: i32,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RemoteSyncGameSummary {
  pub game_key: String,
  pub release_count: usize,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RemoteSyncReleaseSummary {
  pub game_key: String,
  pub release_id: String,
  pub snapshot_commit: String,
  pub first_party_instance_id: String,
  pub first_party_base_url: String,
  pub published_at: i64,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RemoteSyncReleaseDetail {
  pub game_key: String,
  pub release_id: String,
  pub snapshot_commit: String,
  pub manifest_sha256: String,
  pub manifest_body: String,
  pub first_party_instance_id: String,
  pub first_party_base_url: String,
  pub published_at: i64,
}

#[derive(Serialize, Deserialize)]
pub(super) struct DirectDiscoverResponse {
  pub info: RemoteSyncInfo,
  pub games: Option<Vec<RemoteSyncGameSummary>>,
  pub releases: Option<Vec<RemoteSyncReleaseSummary>>,
  pub release: Option<RemoteSyncReleaseDetail>,
}

pub(super) async fn discover_remote_source(
  State(client): State<HTTPClient>, Json(req): Json<DirectDiscoverRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let base_url = normalize_base_url(&req.base_url)?;
  let token = normalized_sync_token(req.sync_token.as_deref());
  let info: RemoteSyncInfo = fetch_remote_json(&client, &base_url, "/info", token).await?;

  let response = if let Some(game_key) = req
    .game_key
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
  {
    if let Some(release_id) = req
      .release_id
      .as_deref()
      .map(str::trim)
      .filter(|value| !value.is_empty())
    {
      let release: RemoteSyncReleaseDetail = fetch_remote_json(
        &client,
        &base_url,
        &format!("/games/{game_key}/releases/{release_id}"),
        token,
      )
      .await?;
      DirectDiscoverResponse {
        info,
        games: None,
        releases: None,
        release: Some(release),
      }
    } else {
      let releases: Vec<RemoteSyncReleaseSummary> =
        fetch_remote_json(&client, &base_url, &format!("/games/{game_key}"), token).await?;
      DirectDiscoverResponse {
        info,
        games: None,
        releases: Some(releases),
        release: None,
      }
    }
  } else {
    let games: Vec<RemoteSyncGameSummary> =
      fetch_remote_json(&client, &base_url, "/games", token).await?;
    DirectDiscoverResponse {
      info,
      games: Some(games),
      releases: None,
      release: None,
    }
  };

  Ok(Json(response))
}

pub(super) async fn import_remote_release(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Json(req): Json<DirectImportRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let base_url = normalize_base_url(&req.base_url)?;
  let sync_token = normalized_sync_token(req.sync_token.as_deref()).map(str::to_owned);
  let game_key = req.game_key.trim();
  let release_id = req.release_id.trim();
  if game_key.is_empty() || release_id.is_empty() {
    return Err(ResponseError::BadRequest(
      "game key and release id are required".to_owned(),
    ));
  }
  if game::get_by_sync_key(&state.db.conn, game_key)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "a local game with the same sync key already exists".to_owned(),
    ));
  }

  let remote_info: RemoteSyncInfo =
    fetch_remote_json(&state.requestor, &base_url, "/info", sync_token.as_deref()).await?;
  if remote_info.protocol_version != 1 {
    return Err(ResponseError::PreconditionFailed(format!(
      "unsupported remote sync protocol version {}",
      remote_info.protocol_version
    )));
  }
  let release: RemoteSyncReleaseDetail = fetch_remote_json(
    &state.requestor,
    &base_url,
    &format!("/games/{game_key}/releases/{release_id}"),
    sync_token.as_deref(),
  )
  .await?;
  let manifest: manifest::ReleaseManifest = r2s_config::toml::from_str(&release.manifest_body)
    .map_err(|err| ResponseError::PreconditionFailed(format!("invalid release manifest: {err}")))?;
  if sha256sum_str(&release.manifest_body) != release.manifest_sha256 {
    return Err(ResponseError::Conflict(
      "remote release manifest hash does not match the manifest body".to_owned(),
    ));
  }
  validate_release_detail(game_key, release_id, &release, &manifest)?;
  if manifest.game.host_type != "game" {
    return Err(ResponseError::PreconditionFailed(
      "only archived games are supported by direct sync import in this phase".to_owned(),
    ));
  }

  let job_id = nanoid::nanoid!();
  let workspace = sync::job_workspace_dir(&state.config.bucket, &job_id)
    .map_err(|err| ResponseError::InternalServerError(err.to_string()))?;
  let repo_dir = workspace.join("repo");
  create_dir_all(&workspace).await?;

  if let Err(err) = fetch_release_repository(
    &repo_dir,
    &base_url,
    sync_token.as_deref(),
    game_key,
    release_id,
    &release.snapshot_commit,
  )
  .await
  {
    remove_dir_all(&workspace).await.ok();
    return Err(err);
  }

  if let Err(err) = download_release_media(
    &state.requestor,
    &state.db,
    &state.media,
    &base_url,
    sync_token.as_deref(),
    &manifest.assets.media_hashes,
    token.id,
  )
  .await
  {
    remove_dir_all(&workspace).await.ok();
    return Err(err);
  }

  let bucket_name = pick_local_bucket_name(&state.bucket, game_key);
  let final_repo_path = state.bucket.path().join(&bucket_name);
  rename(&repo_dir, &final_repo_path).await.map_err(|err| {
    ResponseError::InternalServerError(format!("failed to finalize imported repository: {err}"))
  })?;

  let imported = finalize_import(
    &state,
    token.id,
    &bucket_name,
    &remote_info,
    release,
    manifest,
  )
  .await;
  remove_dir_all(&workspace).await.ok();

  match imported {
    Ok((game, _release)) => Ok(Json(DirectImportResponse {
      game_id: game.id,
      game_key: game.sync_key.unwrap_or_else(|| game_key.to_owned()),
      release_id: release_id.to_owned(),
      bucket: game.bucket.unwrap_or(bucket_name),
    })),
    Err(err) => {
      remove_dir_all(final_repo_path).await.ok();
      Err(err)
    }
  }
}

async fn finalize_import(
  state: &GlobalState, importer_id: i64, bucket_name: &str, remote_info: &RemoteSyncInfo,
  release: RemoteSyncReleaseDetail, manifest: manifest::ReleaseManifest,
) -> Result<(game::Model, game_release::Model), ResponseError> {
  let txn = state.db.conn.begin().await?;
  let published_at = DateTime::from_timestamp(release.published_at, 0).ok_or(
    ResponseError::PreconditionFailed("invalid release published timestamp".to_owned()),
  )?;

  let game = game::create(
    &txn,
    build_imported_game(importer_id, bucket_name, &manifest),
  )
  .await?;
  import_challenges(&txn, &state.bucket, bucket_name, &game, &manifest).await?;

  let local_release = game_release::create(
    &txn,
    game_release::Model {
      id: 0,
      game_id: game.id,
      game_key: manifest.game_key.clone(),
      release_id: manifest.release_id.clone(),
      snapshot_commit: manifest.snapshot_commit.clone(),
      manifest_sha256: release.manifest_sha256.clone(),
      manifest_body: release.manifest_body.clone(),
      origin_role: game_release::OriginRole::Mirror,
      first_party_instance_id: release.first_party_instance_id.clone(),
      first_party_base_url: release.first_party_base_url.clone(),
      published_at,
      created_at: Utc::now(),
    },
  )
  .await?;

  game_remote_sync::create(
    &txn,
    game_remote_sync::Model {
      game_id: game.id,
      state: game_remote_sync::RemoteGameState::MirrorLocked,
      current_release_id: manifest.release_id.clone(),
      snapshot_commit: manifest.snapshot_commit.clone(),
      manifest_sha256: release.manifest_sha256,
      manifest_body: release.manifest_body,
      first_party_instance_id: release.first_party_instance_id,
      first_party_base_url: release.first_party_base_url,
      selected_upstream_instance_id: remote_info.instance_id.clone(),
      selected_upstream_base_url: remote_info.base_url.clone(),
      last_synced_at: Utc::now(),
      detached_at: None,
      detached_by: None,
    },
  )
  .await?;

  txn.commit().await?;
  Ok((game, local_release))
}

async fn import_challenges(
  txn: &sea_orm::DatabaseTransaction, bucket: &Bucket, bucket_name: &str, game: &game::Model,
  manifest: &manifest::ReleaseManifest,
) -> Result<(), ResponseError> {
  let game_bucket = bucket.at(bucket_name).await?;
  let manifest_challenges: BTreeMap<_, _> = manifest
    .challenges
    .iter()
    .map(|challenge| (challenge.key.as_str(), challenge))
    .collect();
  let bucket_dirs = list_challenge_dirs(&game_bucket).await?;
  if bucket_dirs.len() != manifest_challenges.len() {
    return Err(ResponseError::PreconditionFailed(
      "challenge directories do not match the release manifest".to_owned(),
    ));
  }

  for bucket_key in bucket_dirs {
    let Some(manifest_challenge) = manifest_challenges.get(bucket_key.as_str()) else {
      return Err(ResponseError::PreconditionFailed(format!(
        "challenge `{bucket_key}` is missing from the release manifest"
      )));
    };
    let challenge_bucket = game_bucket.at(&bucket_key).await?;
    validate_import_env(&challenge_bucket).await?;
    let config = challenge_bucket.config().await?;
    let challenge = challenge::create(
      txn,
      challenge::Model {
        id: 0,
        name: config.name,
        updated_at: Utc::now(),
        content: Some(challenge_bucket.description().await?),
        hidden: manifest_challenge.hidden,
        game_id: game.id,
        tag: serde_json::from_value(serde_json::to_value(config.tag).map_err(ResponseError::from)?)
          .map_err(ResponseError::from)?,
        score_rule: serde_json::from_value(
          serde_json::to_value(config.score_rule).map_err(ResponseError::from)?,
        )
        .map_err(ResponseError::from)?,
        score: manifest_challenge.score,
        display_order: manifest_challenge.order,
        bucket: Some(bucket_key.clone()),
        ref_id: None,
        release_at: manifest_challenge.release_at,
        archive_at: manifest_challenge.archive_at,
      },
    )
    .await?;
    for bucket_hint in challenge_bucket.hints().await?.hints {
      hint::create(
        txn,
        hint::Model {
          id: 0,
          created_at: Utc::now(),
          challenge_id: challenge.id,
          content: bucket_hint.content,
          cost: bucket_hint.cost,
        },
      )
      .await?;
    }
  }
  Ok(())
}

fn build_imported_game(
  importer_id: i64, bucket_name: &str, manifest: &manifest::ReleaseManifest,
) -> game::Model {
  let host_type = match manifest.game.host_type.as_str() {
    "game" => game::HostType::Game,
    "training" => game::HostType::Training,
    _ => game::HostType::Game,
  };
  game::Model {
    id: 0,
    updated_at: Utc::now(),
    name: manifest.game.name.clone(),
    brief: manifest.game.brief.clone(),
    introduction_id: None,
    start_at: manifest.game.start_at,
    end_at: manifest.game.end_at,
    register_at: manifest.game.register_at,
    archive_at: manifest.game.archive_at,
    hidden: true,
    offline: false,
    frozen: false,
    host_type,
    team_size: manifest.game.team_size,
    access_policy: game::AccessPolicy {
      restrict: false,
      institutes: vec![],
      sync: manifest.game.sync_policy,
    },
    archive_policy: game::ArchivePolicy {
      challenge: game::ArchivePolicyChallenge {
        show_answer: manifest.game.show_answer_after_archive,
        show_hints: manifest.game.show_hints_after_archive,
      },
    },
    cover: manifest.game.cover_value.clone(),
    logo: manifest.game.logo_value.clone(),
    enable_audit: false,
    can_register_after_started: manifest.game.can_register_after_started,
    award_rate: 0,
    award_rates: Some(game::AwardRates(vec![0, 0, 0])),
    admins: game::Admins(vec![importer_id]),
    weight: manifest.game.weight,
    bucket: Some(bucket_name.to_owned()),
    token: Some(nanoid::nanoid!()),
    sync_key: Some(manifest.game_key.clone()),
    sync_token: Some(nanoid::nanoid!()),
    timeline_presets: None,
    node_selector: None,
    traffic: None,
    lifecycle: None,
    hammer_policy: game::HammerPolicy::default(),
  }
}

async fn fetch_release_repository(
  repo_dir: &Path, base_url: &str, sync_token: Option<&str>, game_key: &str, release_id: &str,
  snapshot_commit: &str,
) -> Result<(), ResponseError> {
  let repo_dir_str = repo_dir.to_string_lossy().to_string();
  run_git(None, sync_token, &["init".to_owned(), repo_dir_str.clone()]).await?;
  let repo_url = format!("{base_url}/api/sync/v1/games/{game_key}/releases/{release_id}/repo");
  let release_ref = manifest::release_ref(release_id);
  run_git(
    Some(repo_dir),
    sync_token,
    &[
      "fetch".to_owned(),
      "--no-tags".to_owned(),
      repo_url,
      format!("{release_ref}:{release_ref}"),
    ],
  )
  .await?;
  run_git(
    Some(repo_dir),
    sync_token,
    &[
      "checkout".to_owned(),
      "--detach".to_owned(),
      snapshot_commit.to_owned(),
    ],
  )
  .await?;
  let git = Git::try_open(repo_dir).await?;
  let head = git.get_head().await?;
  if head != snapshot_commit {
    return Err(ResponseError::Conflict(
      "fetched repository head does not match the release snapshot".to_owned(),
    ));
  }
  Ok(())
}

async fn download_release_media(
  client: &HTTPClient, db: &Database, media_store: &Media, base_url: &str,
  sync_token: Option<&str>, media_hashes: &[String], uploader_id: i64,
) -> Result<(), ResponseError> {
  for hash in media_hashes {
    if media::get_by_hash(&db.conn, hash).await?.is_some() {
      continue;
    }
    let (status, body) =
      fetch_remote_body(client, base_url, &format!("/media/{hash}"), sync_token).await?;
    if !status.is_success() {
      let body = to_bytes(body, usize::MAX).await.map_err(|err| {
        ResponseError::BadRequest(format!("failed to read remote media response: {err}"))
      })?;
      return Err(ResponseError::PreconditionFailed(format!(
        "failed to download media {hash}: {}",
        String::from_utf8_lossy(&body)
      )));
    }
    let reader = StreamReader::new(body.into_data_stream().map_err(std::io::Error::other));
    let model = media_store.save(reader).await?;
    if model.hash != *hash {
      return Err(ResponseError::Conflict(format!(
        "downloaded media hash mismatch: expected {hash}, got {}",
        model.hash
      )));
    }
    if media::get_by_hash(&db.conn, &model.hash).await?.is_none() {
      media::create(
        &db.conn,
        media::Model {
          id: 0,
          hash: model.hash,
          uploader_id,
        },
      )
      .await?;
    }
  }
  Ok(())
}

async fn fetch_remote_body(
  client: &HTTPClient, base_url: &str, path: &str, token: Option<&str>,
) -> Result<(axum::http::StatusCode, Body), ResponseError> {
  let uri = Uri::try_from(format!("{base_url}/api/sync/v1{path}"))
    .map_err(|err| ResponseError::BadRequest(format!("invalid remote sync uri: {err}")))?;
  let mut request = Request::builder().method(Method::GET).uri(uri);
  if let Some(token) = token {
    request = request.header(
      AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|err| ResponseError::BadRequest(format!("invalid sync token header: {err}")))?,
    );
  }
  let request = request.body(Body::empty()).map_err(|err| {
    ResponseError::InternalServerError(format!("failed to build sync request: {err}"))
  })?;
  let response = client.request(request).await.map_err(|err| {
    ResponseError::BadRequest(format!("failed to reach remote sync endpoint: {err}"))
  })?;
  let status = response.status();
  Ok((status, Body::new(response.into_body())))
}

async fn fetch_remote_json<T: serde::de::DeserializeOwned>(
  client: &HTTPClient, base_url: &str, path: &str, token: Option<&str>,
) -> Result<T, ResponseError> {
  let (status, body) = fetch_remote_body(client, base_url, path, token).await?;
  let body = to_bytes(body, usize::MAX).await.map_err(|err| {
    ResponseError::BadRequest(format!("failed to read remote sync response: {err}"))
  })?;
  if !status.is_success() {
    let body_text = String::from_utf8_lossy(&body);
    return Err(ResponseError::PreconditionFailed(format!(
      "remote sync endpoint returned {status}: {body_text}"
    )));
  }
  serde_json::from_slice(&body).map_err(ResponseError::from)
}

fn validate_release_detail(
  game_key: &str, release_id: &str, release: &RemoteSyncReleaseDetail,
  manifest: &manifest::ReleaseManifest,
) -> Result<(), ResponseError> {
  if release.game_key != game_key
    || release.release_id != release_id
    || manifest.game_key != game_key
    || manifest.release_id != release_id
  {
    return Err(ResponseError::Conflict(
      "remote release metadata does not match the requested game key or release id".to_owned(),
    ));
  }
  if release.snapshot_commit != manifest.snapshot_commit {
    return Err(ResponseError::Conflict(
      "remote release metadata does not match the manifest snapshot".to_owned(),
    ));
  }
  Ok(())
}

fn normalized_sync_token(sync_token: Option<&str>) -> Option<&str> {
  sync_token.map(str::trim).filter(|token| !token.is_empty())
}

fn normalize_base_url(base_url: &str) -> Result<String, ResponseError> {
  let base_url = base_url.trim().trim_end_matches('/');
  if base_url.is_empty() {
    return Err(ResponseError::BadRequest(
      "remote base url can not be empty".to_owned(),
    ));
  }
  if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
    return Err(ResponseError::BadRequest(
      "remote base url must start with http:// or https://".to_owned(),
    ));
  }
  Ok(base_url.to_owned())
}

fn pick_local_bucket_name(bucket: &Bucket, game_key: &str) -> String {
  let base = sanitize_bucket_seed(game_key);
  if !bucket.path().join(&base).exists() {
    return base;
  }
  for index in 1..=9999 {
    let candidate = format!("{base}_{index}");
    if !bucket.path().join(&candidate).exists() {
      return candidate;
    }
  }
  format!("{}_{}", base, &nanoid::nanoid!()[..8])
}

fn sanitize_bucket_seed(value: &str) -> String {
  let normalized = deunicode_with_tofu(value, "_")
    .trim()
    .to_owned()
    .to_snake_case()
    .chars()
    .map(|ch| {
      if ch.is_ascii_alphanumeric() || ch == '_' {
        ch
      } else {
        '_'
      }
    })
    .collect::<String>();
  let normalized = normalized.trim_matches('_');
  let normalized = if normalized.is_empty() {
    "synced_game".to_owned()
  } else {
    normalized.to_owned()
  };
  if normalized.len() > 72 {
    normalized[..72].to_owned()
  } else {
    normalized
  }
}

async fn list_challenge_dirs(
  game_bucket: &r2s_bucket::game::GameBucket,
) -> Result<Vec<String>, ResponseError> {
  let challenges_root = game_bucket.git.path().join("challenges");
  let mut entries = read_dir(&challenges_root).await.map_err(|err| {
    ResponseError::InternalServerError(format!(
      "failed to read imported challenge directory {}: {err}",
      challenges_root.display()
    ))
  })?;
  let mut result = Vec::new();
  while let Some(entry) = entries.next_entry().await? {
    if !entry.file_type().await?.is_dir() {
      continue;
    }
    let name = entry.file_name().to_string_lossy().to_string();
    if name.starts_with('.') {
      continue;
    }
    result.push(name);
  }
  result.sort();
  Ok(result)
}

async fn validate_import_env(
  challenge_bucket: &r2s_bucket::challenge::ChallengeBucket,
) -> Result<(), ResponseError> {
  let Some(env) = challenge_bucket.env().await? else {
    return Ok(());
  };
  let mut ports = std::collections::HashSet::new();
  for image in env.images {
    if let Some(port) = image.port
      && !ports.insert(port)
    {
      return Err(ResponseError::PreconditionFailed(format!(
        "challenge `{}` has conflicting ports in env.toml",
        challenge_bucket.name
      )));
    }
  }
  Ok(())
}

async fn run_git(
  current_dir: Option<&Path>, sync_token: Option<&str>, args: &[String],
) -> Result<(), ResponseError> {
  let mut cmd = Command::new("git");
  if let Some(current_dir) = current_dir {
    cmd.current_dir(current_dir);
  }
  if let Some(sync_token) = sync_token {
    cmd.arg("-c").arg(format!(
      "http.extraHeader=Authorization: Bearer {sync_token}"
    ));
  }
  cmd.args(args);
  let output = cmd.output().await?;
  if output.status.success() {
    Ok(())
  } else {
    Err(ResponseError::PreconditionFailed(
      String::from_utf8_lossy(&output.stderr).to_string(),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::{normalize_base_url, sanitize_bucket_seed, validate_release_detail};
  use crate::sync::manifest::{ChallengeManifest, GameManifest, ManifestAssets, ReleaseManifest};

  #[test]
  fn normalize_base_url_trims_trailing_slashes() {
    assert_eq!(
      normalize_base_url(" https://example.com/ ").expect("normalize base url"),
      "https://example.com"
    );
  }

  #[test]
  fn normalize_base_url_rejects_invalid_scheme() {
    let err = normalize_base_url("example.com").expect_err("invalid base url should fail");
    assert!(format!("{err}").contains("http:// or https://"));
  }

  #[test]
  fn sanitize_bucket_seed_produces_stable_ascii_bucket_name() {
    assert_eq!(
      sanitize_bucket_seed("Example Game 2024"),
      "example_game_2024"
    );
  }

  #[test]
  fn validate_release_detail_rejects_mismatched_snapshot() {
    let err = validate_release_detail(
      "game_key",
      "release_id",
      &super::RemoteSyncReleaseDetail {
        game_key: "game_key".to_owned(),
        release_id: "release_id".to_owned(),
        snapshot_commit: "deadbeef".to_owned(),
        manifest_sha256: "hash".to_owned(),
        manifest_body: String::new(),
        first_party_instance_id: "instance".to_owned(),
        first_party_base_url: "https://example.com".to_owned(),
        published_at: 0,
      },
      &ReleaseManifest {
        spec_version: 1,
        kind: "release".to_owned(),
        game_key: "game_key".to_owned(),
        release_id: "release_id".to_owned(),
        snapshot_commit: "another".to_owned(),
        published_at: chrono::DateTime::UNIX_EPOCH,
        first_party_instance_id: "instance".to_owned(),
        game: GameManifest {
          name: "name".to_owned(),
          brief: "brief".to_owned(),
          host_type: "game".to_owned(),
          start_at: chrono::DateTime::UNIX_EPOCH,
          end_at: chrono::DateTime::UNIX_EPOCH,
          register_at: chrono::DateTime::UNIX_EPOCH,
          archive_at: chrono::DateTime::UNIX_EPOCH,
          team_size: 1,
          weight: 1,
          sync_policy: 0,
          can_register_after_started: true,
          cover_kind: None,
          cover_value: None,
          logo_kind: None,
          logo_value: None,
          show_answer_after_archive: false,
          show_hints_after_archive: false,
        },
        challenges: vec![ChallengeManifest {
          key: "challenge".to_owned(),
          order: 1,
          hidden: false,
          score: 100,
          release_at: None,
          archive_at: None,
        }],
        assets: ManifestAssets::default(),
      },
    )
    .expect_err("snapshot mismatch should fail");

    assert!(format!("{err}").contains("manifest snapshot"));
  }
}
