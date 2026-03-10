use axum::{
  Json,
  extract::{Path, State},
  response::IntoResponse,
};
use chrono::Utc;
use r2s_cache::Cache;
use r2s_config::GlobalConfig;
use r2s_database::game_registry_source;
use r2s_migrator::Database;
use serde::Deserialize;

use crate::{sync::registry, traits::ResponseError};

#[derive(Deserialize)]
pub(super) struct RegistrySourceRequest {
  pub name: String,
  pub git_url: String,
  pub branch: String,
  pub enabled: bool,
  pub priority: i32,
  pub publish_enabled: bool,
  pub private_source: bool,
}

pub(super) async fn list_registry_sources(
  State(ref db): State<Database>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(game_registry_source::get_list(&db.conn).await?))
}

pub(super) async fn create_registry_source(
  State(ref db): State<Database>, Json(req): Json<RegistrySourceRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let now = Utc::now();
  let source = game_registry_source::create(
    &db.conn,
    game_registry_source::Model {
      id: 0,
      name: req.name,
      git_url: req.git_url,
      branch: req.branch,
      enabled: req.enabled,
      priority: req.priority,
      publish_enabled: req.publish_enabled,
      private_source: req.private_source,
      last_fetched_at: None,
      last_error: None,
      created_at: now,
      updated_at: now,
    },
  )
  .await?;
  Ok(Json(source))
}

pub(super) async fn update_registry_source(
  State(config): State<GlobalConfig>, State(ref db): State<Database>, Path(source_id): Path<i64>,
  Json(req): Json<RegistrySourceRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let previous = game_registry_source::get(&db.conn, source_id)
    .await?
    .ok_or(ResponseError::NotFound(
      "registry source not found".to_owned(),
    ))?;
  let should_reset_cache = previous.git_url != req.git_url || previous.branch != req.branch;
  if should_reset_cache {
    registry::remove_registry_source_cache(&config.bucket, source_id)
      .await
      .map_err(|err| ResponseError::InternalServerError(err.to_string()))?;
  }
  let source = game_registry_source::update(
    &db.conn,
    game_registry_source::Model {
      id: previous.id,
      name: req.name,
      git_url: req.git_url,
      branch: req.branch,
      enabled: req.enabled,
      priority: req.priority,
      publish_enabled: req.publish_enabled,
      private_source: req.private_source,
      last_fetched_at: if should_reset_cache {
        None
      } else {
        previous.last_fetched_at
      },
      last_error: if should_reset_cache {
        None
      } else {
        previous.last_error
      },
      created_at: previous.created_at,
      updated_at: Utc::now(),
    },
  )
  .await?;
  Ok(Json(source))
}

pub(super) async fn delete_registry_source(
  State(config): State<GlobalConfig>, State(ref db): State<Database>, State(cache): State<Cache>,
  Path(source_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  let existing = game_registry_source::get(&db.conn, source_id)
    .await?
    .ok_or(ResponseError::NotFound(
      "registry source not found".to_owned(),
    ))?;
  registry::remove_registry_source_cache(&config.bucket, source_id)
    .await
    .map_err(|err| ResponseError::InternalServerError(err.to_string()))?;
  game_registry_source::delete(&db.conn, source_id).await?;
  cache.at("sync-source").del(existing.id).await.ok();
  Ok(())
}

pub(super) async fn fetch_registry_source(
  State(config): State<GlobalConfig>, State(ref db): State<Database>, Path(source_id): Path<i64>,
) -> Result<impl IntoResponse, ResponseError> {
  let source = game_registry_source::get(&db.conn, source_id)
    .await?
    .ok_or(ResponseError::NotFound(
      "registry source not found".to_owned(),
    ))?;
  let fetch_result = registry::fetch_registry_source(&config.bucket, &source).await;
  let updated_source = game_registry_source::update(
    &db.conn,
    game_registry_source::Model {
      last_fetched_at: fetch_result.as_ref().ok().map(|_| Utc::now()),
      last_error: fetch_result.as_ref().err().map(|err| err.to_string()),
      updated_at: Utc::now(),
      ..source
    },
  )
  .await?;
  match fetch_result {
    Ok(_) => Ok(Json(updated_source)),
    Err(err) => Err(ResponseError::PreconditionFailed(err.to_string())),
  }
}
