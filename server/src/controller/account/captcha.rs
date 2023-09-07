use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router,
};

use crate::{
    cache::manager::RedisPool, captcha, controller::GlobalState,
    entity::platform_info::Model as PlatformInfoModel,
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new().route("/", get(get_captcha))
}

async fn get_captcha(
    State(mut cache): State<RedisPool>,
    Extension(platform_info): Extension<PlatformInfoModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(captcha) = platform_info.captcha {
        if !captcha.enabled {
            return Ok(Json(captcha::Captcha {
                id: "".to_string(),
                validator: captcha::Validator::None,
                challenge: "".to_string(),
                answer: "".to_string(),
            }));
        }
        Ok(Json(
            captcha::generate_captcha(
                captcha.validator,
                &mut cache,
                captcha.difficulty.unwrap_or(4),
            )
            .await
            .map_err(|err| {
                tracing::error!("query platform info from cache failed: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "encountered cache error")
            })?,
        ))
    } else {
        Err((StatusCode::NOT_FOUND, "not found"))
    }
}
