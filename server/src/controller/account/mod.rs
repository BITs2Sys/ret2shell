mod captcha;

use axum::{
    extract::State, http::StatusCode, middleware::from_fn, response::IntoResponse, routing::post,
    Extension, Json, Router,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{cache::manager::RedisPool, controller::GlobalState};

use super::layer::auth::{permission_required, Token};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route_layer(from_fn(permission_required!("basic")))
        .route("/login", post(login))
        .nest("/captcha", captcha::router(state))
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    pub account: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn login(
    State(ref db): State<DatabaseConnection>,
    State(ref mut cache): State<RedisPool>,
    Extension(token): Extension<Token>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("login request: {:?}", body);
    Ok(StatusCode::OK)
}
