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
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct NormalizedRegistrySourceRequest {
  name: String,
  git_url: String,
  branch: String,
  enabled: bool,
  priority: i32,
  publish_enabled: bool,
  private_source: bool,
}

impl RegistrySourceRequest {
  fn normalize(self) -> Result<NormalizedRegistrySourceRequest, ResponseError> {
    let name = self.name.trim().to_owned();
    let git_url = self.git_url.trim().to_owned();
    let branch = self.branch.trim().to_owned();
    if name.is_empty() {
      return Err(ResponseError::BadRequest(
        "registry source name can not be empty".to_owned(),
      ));
    }
    if git_url.is_empty() {
      return Err(ResponseError::BadRequest(
        "registry source git url can not be empty".to_owned(),
      ));
    }
    if branch.is_empty() {
      return Err(ResponseError::BadRequest(
        "registry source branch can not be empty".to_owned(),
      ));
    }
    Ok(NormalizedRegistrySourceRequest {
      name,
      git_url,
      branch,
      enabled: self.enabled,
      priority: self.priority,
      publish_enabled: self.publish_enabled,
      private_source: self.private_source,
    })
  }
}

pub(super) async fn list_registry_sources(
  State(ref db): State<Database>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(game_registry_source::get_list(&db.conn).await?))
}

pub(super) async fn create_registry_source(
  State(ref db): State<Database>, Json(req): Json<RegistrySourceRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let req = req.normalize()?;
  ensure_registry_source_unique(&db.conn, &req, None).await?;
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
  let req = req.normalize()?;
  let previous = game_registry_source::get(&db.conn, source_id)
    .await?
    .ok_or(ResponseError::NotFound(
      "registry source not found".to_owned(),
    ))?;
  ensure_registry_source_unique(&db.conn, &req, Some(previous.id)).await?;
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

async fn ensure_registry_source_unique(
  db: &sea_orm::DatabaseConnection, req: &NormalizedRegistrySourceRequest, current_id: Option<i64>,
) -> Result<(), ResponseError> {
  if let Some(existing) = game_registry_source::get_by_name(db, &req.name).await?
    && Some(existing.id) != current_id
  {
    return Err(ResponseError::Conflict(
      "another registry source already uses the same name".to_owned(),
    ));
  }
  if let Some(existing) =
    game_registry_source::get_by_git_url_and_branch(db, &req.git_url, &req.branch).await?
    && Some(existing.id) != current_id
  {
    return Err(ResponseError::Conflict(
      "another registry source already uses the same git url and branch".to_owned(),
    ));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{NormalizedRegistrySourceRequest, RegistrySourceRequest};

  #[test]
  fn normalize_registry_source_request_trims_fields() {
    let normalized = RegistrySourceRequest {
      name: "  official  ".to_owned(),
      git_url: "  https://example.com/repo.git  ".to_owned(),
      branch: "  main  ".to_owned(),
      enabled: true,
      priority: 0,
      publish_enabled: true,
      private_source: false,
    }
    .normalize()
    .expect("normalize source request");

    assert_eq!(
      normalized,
      NormalizedRegistrySourceRequest {
        name: "official".to_owned(),
        git_url: "https://example.com/repo.git".to_owned(),
        branch: "main".to_owned(),
        enabled: true,
        priority: 0,
        publish_enabled: true,
        private_source: false,
      }
    );
  }

  #[test]
  fn normalize_registry_source_request_rejects_empty_required_fields() {
    let err = RegistrySourceRequest {
      name: " ".to_owned(),
      git_url: " ".to_owned(),
      branch: " ".to_owned(),
      enabled: true,
      priority: 0,
      publish_enabled: false,
      private_source: false,
    }
    .normalize()
    .expect_err("empty source request should fail");

    assert!(format!("{err}").contains("registry source name"));
  }
}
