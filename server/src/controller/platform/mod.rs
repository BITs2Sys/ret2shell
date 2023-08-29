use axum::http::StatusCode;
use axum::{response::IntoResponse, routing::get, Router};
use axum::{Extension, Json};
use tracing::error;

use crate::controller::GlobalState;
use crate::entity::platform_info::PlatformInfoModel;

pub fn router() -> Router<GlobalState> {
    Router::new().route("/", get(get_platform_info))
}

async fn get_platform_info(
    Extension(platform_info): Extension<Option<PlatformInfoModel>>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(platform_info) = platform_info {
        Ok(Json(platform_info))
    } else {
        error!("platform info not found");
        Err((StatusCode::INTERNAL_SERVER_ERROR, "encountered cache error"))
    }
}
