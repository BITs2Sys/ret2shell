use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    Extension,
};
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;
use tracing::{debug, error};

use super::auth::Token;
use crate::{
    cache::{manager::RedisPool, platform::Platform},
    entity::user,
};

pub async fn prepare_platform_info(
    State(ref db): State<DatabaseConnection>, State(ref mut pool): State<RedisPool>,
    mut req: Request, next: Next,
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

pub async fn prepare_user_full_info(
    State(ref db): State<DatabaseConnection>, Extension(token): Extension<Token>, mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let user_id = token.id;
    match user::get_user(db, user_id).await {
        Ok(user_full) => {
            req.extensions_mut().insert(user_full);
        }
        Err(DbErr::RecordNotFound(_)) => {}
        Err(err) => {
            error!("failed to get user: {}", err);
        }
    };
    Ok(next.run(req).await)
}

pub async fn prepare_challenge_info(
    State(ref db): State<DatabaseConnection>, Path(challenge_id): Path<i64>, mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match crate::entity::challenge::get_challenge(db, challenge_id).await {
        Ok(challenge) => {
            match crate::entity::game::get_game(db, challenge.game_id).await {
                Ok(Some(game)) => {
                    req.extensions_mut().insert(game);
                }
                Err(err) => {
                    error!("failed to get game: {}", err);
                }
                _ => {}
            };
            req.extensions_mut().insert(challenge);
        }
        Err(err) => {
            error!("failed to get challenge: {}", err);
        }
    };

    Ok(next.run(req).await)
}

pub async fn prepare_game_info_in_path(
    State(ref db): State<DatabaseConnection>, Path(game_id): Path<i64>, mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match crate::entity::game::get_game(db, game_id).await {
        Ok(Some(game)) => {
            req.extensions_mut().insert(game);
        }
        Err(err) => {
            error!("failed to get game: {}", err);
        }
        _ => {}
    };

    Ok(next.run(req).await)
}

#[derive(Deserialize)]
pub struct GameQuery {
    pub game_id: Option<i64>,
}

pub async fn prepare_game_info_in_query(
    State(ref db): State<DatabaseConnection>, Query(query): Query<GameQuery>, mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if let Some(game_id) = query.game_id {
        match crate::entity::game::get_game(db, game_id).await {
            Ok(Some(game)) => {
                debug!("game: {:?}", game);
                req.extensions_mut().insert(game);
            }
            Err(err) => {
                error!("failed to get game: {}", err);
            }
            _ => {}
        };
    }

    Ok(next.run(req).await)
}
