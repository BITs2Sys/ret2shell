use super::auth::Token;
use crate::{
    cache::{manager::RedisPool, platform::Platform},
    entity::user,
};
use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use sea_orm::DatabaseConnection;
use tracing::error;

pub async fn prepare_platform_info<B>(
    State(ref db): State<DatabaseConnection>,
    State(ref mut pool): State<RedisPool>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match Platform::get(pool, db).await {
        Ok(platform_info) => {
            req.extensions_mut().insert(platform_info);
        }
        Err(err) => {
            error!("failed to get platform info: {}", err);
        }
    };
    Ok(next.run(req).await)
}

pub async fn prepare_user_full_info<B>(
    State(ref db): State<DatabaseConnection>,
    Extension(token): Extension<Token>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let user_id = token.id;
    match user::get_user(db, user_id).await {
        Ok(user_full) => {
            req.extensions_mut().insert(user_full);
        }
        Err(err) => {
            error!("failed to get user: {}", err);
        }
    };
    Ok(next.run(req).await)
}

pub async fn prepare_challenge_info<B>(
    State(ref db): State<DatabaseConnection>,
    Path(challenge_id): Path<i64>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match crate::entity::challenge::get_challenge(db, challenge_id).await {
        Ok(challenge) => {
            req.extensions_mut().insert(challenge);
        }
        Err(err) => {
            error!("failed to get challenge: {}", err);
        }
    };

    Ok(next.run(req).await)
}
