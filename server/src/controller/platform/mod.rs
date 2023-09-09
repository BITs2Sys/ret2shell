use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::{extract::Extension, Json};
use axum::{response::IntoResponse, routing::get, Router};
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::cache;
use crate::cache::manager::RedisPool;
use crate::controller::GlobalState;
use crate::entity::config::{self, Model as ConfigModel};

use super::layer::auth::init_token_or_permission_required;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route(
            "/config",
            get(get_platform_config)
                .post(set_platform_config)
                .head(test_platform_init_token),
        )
        .route_layer(from_fn_with_state(
            state.clone(),
            init_token_or_permission_required,
        ))
        .route("/", get(get_platform_info))
}

async fn get_platform_info(
    platform_info: Option<Extension<ConfigModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(platform_info)) = platform_info {
        Ok(Json(platform_info.platform))
    } else {
        error!("platform info not found");
        Err((StatusCode::NOT_FOUND, "platform info not found"))
    }
}

async fn get_platform_config(
    platform_info: Option<Extension<ConfigModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(platform_info)) = platform_info {
        Ok(Json(platform_info))
    } else {
        error!("platform config not found");
        Err((StatusCode::NOT_FOUND, "platform config not found"))
    }
}

async fn set_platform_config(
    State(ref db): State<DatabaseConnection>,
    State(ref mut cache): State<RedisPool>,
    Json(new_model): Json<ConfigModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    config::update_config(db, new_model).await.map_err(|err| {
        error!("failed to update platform error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to update platform error [DbErr]",
        )
    })?;
    cache::platform::Platform::refresh_cache(cache, db)
        .await
        .map_err(|err| {
            error!("failed to update platform error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update platform error [CacheErr]",
            )
        })?;
    Ok(StatusCode::OK)
}

async fn test_platform_init_token() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(StatusCode::OK)
}
