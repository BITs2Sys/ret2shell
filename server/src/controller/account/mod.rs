mod captcha;

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::post,
    Extension, Json, Router,
};
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::layer::auth::{Token, TokenTracker};
use crate::captcha::captcha_protected;
use crate::entity::config::Model as ConfigModel;
use crate::entity::user::{self, Permissions};
use crate::{cache::manager::RedisPool, controller::GlobalState, entity::user::Permission};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
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
    Extension(config): Extension<ConfigModel>,
    Extension(token): Extension<Token>,
    Extension(token_tracker): Extension<TokenTracker>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("login request: {:?}", body);
    if token.id >= 0 {
        return Err((StatusCode::CONFLICT, "you are already logged in"));
    }
    let captcha_config = &config.captcha;
    captcha_protected!(&captcha_config, cache, &body.captcha_id, &body.captcha_answer);

    let user = user::get_user_by_account(db, &body.account)
        .await
        .map_err(|err| {
            warn!("failed to get user by account: {:?}", err);
            match err {
                DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "account does not exist"),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to get user by account",
                ),
            }
        })?;

    match bcrypt::verify(body.password, &user.password.unwrap_or(String::new())) {
        Ok(true) => {
            *(token_tracker.token.lock().await) = Token {
                id: user.id,
                name: user.name,
                permissions: user.permissions,
                ..Default::default()
            };
            token_tracker
                .renew_requested
                .store(true, std::sync::atomic::Ordering::Relaxed);

            Ok(StatusCode::OK)
        }
        Ok(false) => Err((StatusCode::UNAUTHORIZED, "wrong password")),
        Err(err) => {
            warn!("failed to verify password: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to verify password with server error",
            ))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn register(
    State(ref db): State<DatabaseConnection>,
    State(ref mut cache): State<RedisPool>,
    Extension(config): Extension<ConfigModel>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("register request: {:?}", body);
    let captcha_config = &config.captcha;
    captcha_protected!(&captcha_config, cache, &body.captcha_id, &body.captcha_answer);

    match user::get_user_by_account(db, &body.email).await {
        Ok(_) => return Err((StatusCode::CONFLICT, "account already exists")),
        _ => (),
    }

    let password = bcrypt::hash(body.password, bcrypt::DEFAULT_COST).map_err(|err| {
        warn!("failed to hash password: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to hash password")
    })?;

    let new_user = user::Model {
        name: body.name,
        password: Some(password),
        email: Some(body.email),
        permissions: Permissions(vec![Permission::Basic]),
        hidden: false,
        banned: false,
        ..Default::default()
    };

    Ok(StatusCode::OK)
}
