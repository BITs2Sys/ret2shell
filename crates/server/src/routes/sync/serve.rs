use std::collections::BTreeMap;

use axum::{
  Json,
  body::{Body, Bytes},
  extract::{Path, Query, Request, State},
  http::{
    HeaderMap, HeaderValue, StatusCode, Uri,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, RANGE},
  },
  response::IntoResponse,
};
use base64::Engine;
use futures::TryStreamExt;
use r2s_bucket::{Bucket, git::to_pkt_line};
use r2s_config::GlobalConfig;
use r2s_database::{game, game_release, game_remote_sync, media};
use r2s_media::Media;
use r2s_migrator::Database;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::error;

use crate::{
  sync,
  traits::{GlobalState, ResponseError},
};

pub fn router(_state: &GlobalState) -> axum::Router<GlobalState> {
  axum::Router::new()
    .route("/info", axum::routing::get(get_sync_info))
    .route("/games", axum::routing::get(list_sync_games))
    .route(
      "/games/{game_key}",
      axum::routing::get(list_sync_game_releases),
    )
    .route(
      "/games/{game_key}/releases/{release_id}",
      axum::routing::get(get_sync_release_detail),
    )
    .route(
      "/games/{game_key}/releases/{release_id}/repo/info/refs",
      axum::routing::get(get_sync_release_info_refs),
    )
    .route(
      "/games/{game_key}/releases/{release_id}/repo/git-upload-pack",
      axum::routing::post(post_sync_release_upload_pack),
    )
    .route(
      "/games/{game_key}/releases/{release_id}/registry/v2/{*path}",
      axum::routing::get(proxy_sync_registry).head(proxy_sync_registry),
    )
    .route("/media/{hash}", axum::routing::get(get_sync_media))
}

#[derive(Serialize)]
pub(super) struct SyncInfoResponse {
  pub instance_id: String,
  pub base_url: String,
  pub protocol_version: i32,
}

#[derive(Serialize)]
pub(super) struct SyncGameSummaryResponse {
  pub game_key: String,
  pub release_count: usize,
}

#[derive(Serialize)]
pub(super) struct SyncReleaseSummaryResponse {
  pub game_key: String,
  pub release_id: String,
  pub snapshot_commit: String,
  pub first_party_instance_id: String,
  pub first_party_base_url: String,
  #[serde(with = "chrono::serde::ts_seconds")]
  pub published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub(super) struct SyncReleaseDetailResponse {
  pub game_key: String,
  pub release_id: String,
  pub snapshot_commit: String,
  pub manifest_sha256: String,
  pub manifest_body: String,
  pub first_party_instance_id: String,
  pub first_party_base_url: String,
  #[serde(with = "chrono::serde::ts_seconds")]
  pub published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Deserialize)]
pub(super) struct InfoRefsQuery {
  pub service: String,
}

impl InfoRefsQuery {
  fn service_trimmed(&self) -> String {
    self.service.trim_start_matches("git-").to_owned()
  }
}

pub(super) async fn get_sync_info(
  State(config): State<GlobalConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  let base_url = config
    .server
    .as_ref()
    .ok_or(ResponseError::PreconditionFailed(
      "server configuration not found".to_owned(),
    ))?
    .external_origin();
  let instance_id = sync::instance_id(&config.bucket)
    .await
    .map_err(|err| ResponseError::InternalServerError(err.to_string()))?;
  Ok(Json(SyncInfoResponse {
    instance_id,
    base_url,
    protocol_version: 1,
  }))
}

pub(super) async fn list_sync_games(
  State(ref db): State<Database>, headers: HeaderMap,
) -> Result<impl IntoResponse, ResponseError> {
  let token = bearer_token(&headers);
  let releases = game_release::get_list(&db.conn).await?;
  let mut games = BTreeMap::<String, usize>::new();
  for release in releases {
    let Some(game) = game::get(&db.conn, release.game_id).await? else {
      continue;
    };
    if !release_visible_to_token(&db.conn, &game, token.as_deref()).await? {
      continue;
    }
    *games.entry(release.game_key).or_default() += 1;
  }
  Ok(Json(
    games
      .into_iter()
      .map(|(game_key, release_count)| SyncGameSummaryResponse {
        game_key,
        release_count,
      })
      .collect::<Vec<_>>(),
  ))
}

pub(super) async fn list_sync_game_releases(
  State(ref db): State<Database>, Path(game_key): Path<String>, headers: HeaderMap,
) -> Result<impl IntoResponse, ResponseError> {
  let token = bearer_token(&headers);
  let releases = game_release::get_list_by_game_key(&db.conn, &game_key).await?;
  let mut result = Vec::new();
  for release in releases {
    let Some(game) = game::get(&db.conn, release.game_id).await? else {
      continue;
    };
    if !release_visible_to_token(&db.conn, &game, token.as_deref()).await? {
      continue;
    }
    result.push(SyncReleaseSummaryResponse {
      game_key: release.game_key,
      release_id: release.release_id,
      snapshot_commit: release.snapshot_commit,
      first_party_instance_id: release.first_party_instance_id,
      first_party_base_url: release.first_party_base_url,
      published_at: release.published_at,
    });
  }
  Ok(Json(result))
}

pub(super) async fn get_sync_release_detail(
  State(ref db): State<Database>, Path((game_key, release_id)): Path<(String, String)>,
  headers: HeaderMap,
) -> Result<impl IntoResponse, ResponseError> {
  let (game, release) = get_accessible_release(
    &db.conn,
    &game_key,
    &release_id,
    bearer_token(&headers).as_deref(),
  )
  .await?;
  let _ = game;
  Ok(Json(SyncReleaseDetailResponse {
    game_key: release.game_key,
    release_id: release.release_id,
    snapshot_commit: release.snapshot_commit,
    manifest_sha256: release.manifest_sha256,
    manifest_body: release.manifest_body,
    first_party_instance_id: release.first_party_instance_id,
    first_party_base_url: release.first_party_base_url,
    published_at: release.published_at,
  }))
}

pub(super) async fn get_sync_release_info_refs(
  State(ref db): State<Database>, State(ref bucket): State<Bucket>,
  Path((game_key, release_id)): Path<(String, String)>, Query(query): Query<InfoRefsQuery>,
  headers: HeaderMap, body: Body,
) -> Result<impl IntoResponse, ResponseError> {
  let service = query.service_trimmed();
  if service != "upload-pack" {
    return Err(ResponseError::BadRequest("invalid git service".to_owned()));
  }
  let (game, release) = get_accessible_release(
    &db.conn,
    &game_key,
    &release_id,
    bearer_token(&headers).as_deref(),
  )
  .await?;
  let protocol = get_protocol(&headers)?;
  let game_bucket = bucket
    .at(
      game
        .bucket
        .as_ref()
        .ok_or(ResponseError::PreconditionFailed(
          "game bucket not found".to_owned(),
        ))?,
    )
    .await?;
  ensure_release_ref_is_available(&game_bucket, &release).await?;
  let stream_reader = StreamReader::new(body.into_data_stream().map_err(std::io::Error::other));
  let stdout = game_bucket
    .git
    .info_refs_upload_release_only(
      protocol,
      &crate::sync::manifest::release_ref(&release.release_id),
      stream_reader,
    )
    .await
    .map_err(|err| {
      error!(error=?err, "failed to run sync git info refs");
      ResponseError::InternalServerError("failed to run git rpc".to_owned())
    })?;
  let stdout_stream = ReaderStream::new(stdout);
  let mut response_headers = HeaderMap::new();
  response_headers.insert(
    CONTENT_TYPE,
    HeaderValue::from_str("application/x-git-upload-pack-advertisement").unwrap(),
  );
  let header = tokio_stream::once(Ok(Bytes::from(format!(
    "{}0000",
    to_pkt_line("# service=git-upload-pack\n")
  ))));
  let stream = header.chain(stdout_stream);
  Ok((StatusCode::OK, response_headers, Body::from_stream(stream)))
}

pub(super) async fn post_sync_release_upload_pack(
  State(ref db): State<Database>, State(ref bucket): State<Bucket>,
  Path((game_key, release_id)): Path<(String, String)>, headers: HeaderMap, body: Body,
) -> Result<impl IntoResponse, ResponseError> {
  let (game, release) = get_accessible_release(
    &db.conn,
    &game_key,
    &release_id,
    bearer_token(&headers).as_deref(),
  )
  .await?;
  let (protocol, response_headers) = prepare_git_rpc_headers("upload-pack", &headers)?;
  let game_bucket = bucket
    .at(
      game
        .bucket
        .as_ref()
        .ok_or(ResponseError::PreconditionFailed(
          "game bucket not found".to_owned(),
        ))?,
    )
    .await?;
  ensure_release_ref_is_available(&game_bucket, &release).await?;
  let stream_reader = StreamReader::new(body.into_data_stream().map_err(std::io::Error::other));
  let stdout = game_bucket
    .git
    .upload_pack_release_only(
      protocol,
      &crate::sync::manifest::release_ref(&release.release_id),
      stream_reader,
    )
    .await
    .map_err(|err| {
      error!(error=?err, "failed to run sync git upload pack");
      ResponseError::InternalServerError("failed to run git rpc".to_owned())
    })?;
  Ok((
    StatusCode::OK,
    response_headers,
    Body::from_stream(ReaderStream::new(stdout)),
  ))
}

pub(super) async fn get_sync_media(
  State(ref db): State<Database>, State(ref media_store): State<Media>, Path(hash): Path<String>,
  headers: HeaderMap,
) -> Result<impl IntoResponse, ResponseError> {
  let token = bearer_token(&headers);
  if !media_is_accessible(&db.conn, &hash, token.as_deref()).await? {
    return Err(ResponseError::Forbidden(
      "media is not accessible through sync".to_owned(),
    ));
  }
  let model = media::get_by_hash(&db.conn, &hash).await?;
  if model.is_none() {
    return Err(ResponseError::NotFound("media".to_owned()));
  }
  let file = media_store.get(&hash).await?;
  let stream = ReaderStream::new(file);
  let mut response_headers = HeaderMap::new();
  response_headers.insert(
    CONTENT_TYPE,
    media_store
      .get_mime_type(&hash)?
      .parse::<HeaderValue>()
      .map_err(|_| ResponseError::InternalServerError("failed to parse mime type".to_owned()))?,
  );
  Ok((StatusCode::OK, response_headers, Body::from_stream(stream)))
}

pub(super) async fn proxy_sync_registry(
  State(config): State<GlobalConfig>, State(client): State<crate::traits::HTTPClient>,
  State(ref db): State<Database>,
  Path((game_key, release_id, path)): Path<(String, String, String)>, headers: HeaderMap,
  mut req: Request,
) -> Result<impl IntoResponse, ResponseError> {
  let (game, release) = get_accessible_release(
    &db.conn,
    &game_key,
    &release_id,
    bearer_token(&headers).as_deref(),
  )
  .await?;
  let manifest: crate::sync::manifest::ReleaseManifest =
    r2s_config::toml::from_str(&release.manifest_body).map_err(|err| {
      ResponseError::InternalServerError(format!("invalid stored release manifest: {err}"))
    })?;
  let (source_repository, target_kind, reference) = parse_sync_registry_path(&path)?;
  let oci_asset = manifest
    .assets
    .oci_images
    .iter()
    .find(|asset| asset.source_repository == source_repository)
    .ok_or(ResponseError::NotFound("oci asset".to_owned()))?;
  let local_repository = format!(
    "{}/{}",
    game
      .bucket
      .as_deref()
      .ok_or(ResponseError::PreconditionFailed(
        "game bucket not found".to_owned(),
      ))?,
    crate::sync::manifest::split_internal_tag_reference(&oci_asset.internal_tag).0
  );
  let registry_config = config
    .cluster
    .clone()
    .and_then(|cluster| cluster.registry)
    .ok_or(ResponseError::PreconditionFailed(
      "internal registry is not enabled".to_owned(),
    ))?;
  let uri = format!(
    "{}://{}/v2/{}/{}/{}",
    if registry_config.insecure {
      "http"
    } else {
      "https"
    },
    registry_config.server.trim_matches('/'),
    local_repository.trim_matches('/'),
    target_kind,
    reference,
  );
  *req.uri_mut() = Uri::try_from(uri)
    .map_err(|err| ResponseError::BadRequest(format!("invalid relay uri: {err}")))?;
  req.headers_mut().remove(AUTHORIZATION);
  req.headers_mut().remove("host");
  let accept = headers.get(ACCEPT).cloned();
  let range = headers.get(RANGE).cloned();
  req.headers_mut().clear();
  if let Some(accept) = accept {
    req.headers_mut().insert(ACCEPT, accept);
  }
  if let Some(range) = range {
    req.headers_mut().insert(RANGE, range);
  }
  if let Some(username) = registry_config.username
    && let Some(password) = registry_config.password
  {
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
    req.headers_mut().insert(
      AUTHORIZATION,
      HeaderValue::from_str(&format!("Basic {auth}")).map_err(|err| {
        ResponseError::InternalServerError(format!("invalid registry auth header: {err}"))
      })?,
    );
  }
  let response = client
    .request(req)
    .await
    .map_err(|err| ResponseError::BadRequest(format!("registry relay failed: {err}")))?;
  Ok(response.into_response())
}

async fn get_accessible_release(
  db: &sea_orm::DatabaseConnection, game_key: &str, release_id: &str, token: Option<&str>,
) -> Result<(game::Model, game_release::Model), ResponseError> {
  let game = game::get_by_sync_key(db, game_key)
    .await?
    .ok_or(ResponseError::NotFound("game release not found".to_owned()))?;
  let release = game_release::get_by_game_and_release(db, game.id, release_id)
    .await?
    .ok_or(ResponseError::NotFound("game release not found".to_owned()))?;
  if let Some(remote_sync) = game_remote_sync::get(db, game.id).await?
    && remote_sync.state == game_remote_sync::RemoteGameState::Detached
  {
    return Err(ResponseError::Conflict(
      "detached mirrors can no longer serve sync traffic".to_owned(),
    ));
  }
  if !release_visible_to_token(db, &game, token).await? {
    return Err(ResponseError::Forbidden(
      "game release is not accessible through sync".to_owned(),
    ));
  }
  Ok((game, release))
}

async fn ensure_release_ref_is_available(
  game_bucket: &r2s_bucket::game::GameBucket, release: &game_release::Model,
) -> Result<(), ResponseError> {
  let release_ref = crate::sync::manifest::release_ref(&release.release_id);
  let release_ref_oid = game_bucket.git.get_ref(&release_ref).await?;
  if release_ref_oid.as_deref() != Some(release.snapshot_commit.as_str()) {
    return Err(ResponseError::Conflict(
      "requested release ref is missing or no longer matches the recorded snapshot".to_owned(),
    ));
  }
  Ok(())
}

async fn release_visible_to_token(
  db: &sea_orm::DatabaseConnection, game: &game::Model, token: Option<&str>,
) -> Result<bool, ResponseError> {
  if game.access_policy.sync == 2 {
    return Ok(false);
  }
  if let Some(remote_sync) = game_remote_sync::get(db, game.id).await?
    && remote_sync.state == game_remote_sync::RemoteGameState::Detached
  {
    return Ok(false);
  }
  Ok(match game.access_policy.sync {
    0 => true,
    1 => token.is_some_and(|token| game.sync_token.as_deref() == Some(token)),
    _ => false,
  })
}

async fn media_is_accessible(
  db: &sea_orm::DatabaseConnection, hash: &str, token: Option<&str>,
) -> Result<bool, ResponseError> {
  let releases = game_release::get_list(db).await?;
  for release in releases {
    if !release.manifest_body.contains(hash) {
      continue;
    }
    let Some(game) = game::get(db, release.game_id).await? else {
      continue;
    };
    if release_visible_to_token(db, &game, token).await? {
      return Ok(true);
    }
  }
  Ok(false)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
  let authorization = headers.get("Authorization")?.to_str().ok()?;
  authorization.strip_prefix("Bearer ").map(str::to_owned)
}

fn parse_sync_registry_path(path: &str) -> Result<(String, &'static str, String), ResponseError> {
  if let Some((repository, reference)) = path.split_once("/manifests/") {
    return Ok((
      repository.trim_matches('/').to_owned(),
      "manifests",
      reference.to_owned(),
    ));
  }
  if let Some((repository, reference)) = path.split_once("/blobs/") {
    return Ok((
      repository.trim_matches('/').to_owned(),
      "blobs",
      reference.to_owned(),
    ));
  }
  Err(ResponseError::BadRequest(
    "unsupported sync registry path".to_owned(),
  ))
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
  use super::parse_sync_registry_path;

  #[test]
  fn parse_sync_registry_path_supports_manifests_and_blobs() {
    assert_eq!(
      parse_sync_registry_path("game_bucket/web/manifests/sha256:abc").expect("manifest path"),
      (
        "game_bucket/web".to_owned(),
        "manifests",
        "sha256:abc".to_owned()
      )
    );
    assert_eq!(
      parse_sync_registry_path("game_bucket/web/blobs/sha256:def").expect("blob path"),
      (
        "game_bucket/web".to_owned(),
        "blobs",
        "sha256:def".to_owned()
      )
    );
  }

  #[test]
  fn parse_sync_registry_path_rejects_unknown_shapes() {
    assert!(parse_sync_registry_path("game_bucket/web/tags/list").is_err());
  }
}

fn check_git_protocol_safe(protocol: &str) -> bool {
  regex::Regex::new(r"^[0-9a-zA-Z]+=[0-9a-zA-Z]+(:[0-9a-zA-Z]+=[0-9a-zA-Z]+)*$")
    .unwrap()
    .is_match(protocol)
}

fn get_protocol(headers: &HeaderMap) -> Result<String, ResponseError> {
  let protocol = headers.get("Git-Protocol");
  if let Some(protocol) = protocol {
    let protocol = protocol
      .to_str()
      .map_err(|_| ResponseError::BadRequest("invalid git protocol".to_owned()))?;
    if check_git_protocol_safe(protocol) {
      Ok(protocol.to_owned())
    } else {
      Err(ResponseError::BadRequest("invalid git protocol".to_owned()))
    }
  } else {
    Ok("".to_owned())
  }
}

fn prepare_git_rpc_headers(
  service_name: &str, headers: &HeaderMap,
) -> Result<(String, HeaderMap), ResponseError> {
  let expected_content_type = format!("application/x-git-{service_name}-request");
  let content_type = headers.get(CONTENT_TYPE).ok_or(ResponseError::BadRequest(
    "missing content type for git rpc".to_owned(),
  ))?;
  if content_type
    .to_str()
    .map_err(|_| ResponseError::BadRequest("invalid content type for git rpc".to_owned()))?
    != expected_content_type
  {
    return Err(ResponseError::BadRequest(
      "invalid content type for git rpc".to_owned(),
    ));
  }
  let protocol = get_protocol(headers)?;
  let mut response_headers = HeaderMap::new();
  response_headers.insert(
    CONTENT_TYPE,
    HeaderValue::from_str(&format!("application/x-git-{service_name}-result")).unwrap(),
  );
  Ok((protocol, response_headers))
}
