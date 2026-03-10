use axum::{
  Json,
  body::{Body, to_bytes},
  extract::State,
  http::{HeaderValue, Method, Request, Uri, header::AUTHORIZATION},
  response::IntoResponse,
};
use r2s_migrator::Database;
use serde::{Deserialize, Serialize};

use crate::traits::{HTTPClient, ResponseError};

#[derive(Deserialize)]
pub(super) struct DirectDiscoverRequest {
  pub base_url: String,
  pub sync_token: Option<String>,
  pub game_key: Option<String>,
  pub release_id: Option<String>,
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
  State(client): State<HTTPClient>, State(ref _db): State<Database>,
  Json(req): Json<DirectDiscoverRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let base_url = normalize_base_url(&req.base_url)?;
  let token = req
    .sync_token
    .as_deref()
    .map(str::trim)
    .filter(|token| !token.is_empty());
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

async fn fetch_remote_json<T: serde::de::DeserializeOwned>(
  client: &HTTPClient, base_url: &str, path: &str, token: Option<&str>,
) -> Result<T, ResponseError> {
  let uri = Uri::try_from(format!("{base_url}/api/sync/v1{}", path))
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
  let body = to_bytes(Body::new(response.into_body()), usize::MAX)
    .await
    .map_err(|err| {
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

#[cfg(test)]
mod tests {
  use super::normalize_base_url;

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
}
