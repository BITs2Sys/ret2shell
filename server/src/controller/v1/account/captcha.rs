use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::get, Extension, Json, Router,
};

use crate::{
    cache::manager::RedisPool,
    captcha::{self, Validator},
    controller::GlobalState,
    entity::config::Model as ConfigModel,
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", get(get_captcha))
        .route("/cli", get(get_cli_captcha))
}

async fn get_captcha(
    State(mut cache): State<RedisPool>, Extension(platform_info): Extension<ConfigModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let captcha = platform_info.captcha;
    if !captcha.enabled {
        return Ok(Json(captcha::Captcha {
            id: "".to_string(),
            validator: captcha::Validator::None,
            challenge: "".to_string(),
            answer: "".to_string(),
        }));
    }
    Ok(Json(
        captcha::generate_captcha(&captcha.validator, &mut cache, &captcha.difficulty)
            .await
            .map_err(|err| {
                tracing::error!("query platform info from cache failed: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "encountered cache error")
            })?,
    ))
}

async fn get_cli_captcha(
    State(mut cache): State<RedisPool>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json(
        captcha::generate_captcha(&Validator::Pow, &mut cache, &4)
            .await
            .map_err(|err| {
                tracing::error!("query platform info from cache failed: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "encountered cache error")
            })?,
    ))
}
