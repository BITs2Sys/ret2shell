use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::from_fn;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum::{Extension, Json};
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::cache;
use crate::cache::manager::RedisPool;
use crate::controller::GlobalState;
use crate::entity::platform_info::{self, Model as PlatformInfoModel};
use crate::entity::user::Permission;

use super::layer::auth::permission_required;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(set_platform_info))
        .route_layer(from_fn(permission_required!(Permission::Devops)))
        .route("/", get(get_platform_info))
}

async fn get_platform_info(
    platform_info: Option<Extension<PlatformInfoModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(Extension(platform_info)) = platform_info {
        Ok(Json(platform_info))
    } else {
        error!("platform info not found");
        Err((StatusCode::NOT_FOUND, "platform info not found"))
    }
}

async fn set_platform_info(
    State(ref db): State<DatabaseConnection>,
    State(ref mut cache): State<RedisPool>,
    Json(new_model): Json<PlatformInfoModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    platform_info::update_platform_info(db, new_model)
        .await
        .map_err(|err| {
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
