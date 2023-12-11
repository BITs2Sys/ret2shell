use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::Deserialize;
use tracing::error;

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{hint, user::Permission},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_hint).delete(delete_hint))
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Organize,
            Permission::Devops
        )))
        .route("/", get(get_hint_list))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Basic,
            Permission::Verified
        )))
}

#[derive(Deserialize)]
struct HintIDQuery {
    pub hint_id: i64,
}

async fn create_hint(
    State(ref conn): State<DatabaseConnection>, Path(challenge_id): Path<i64>,
    Json(hint): Json<hint::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match hint::create_hint(conn, challenge_id, hint).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "Challenge not found")),
        Err(e) => {
            error!("Failed to create hint: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create hint"))
        }
    }
}

async fn get_hint_list(
    State(ref conn): State<DatabaseConnection>, Path(challenge_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match hint::get_hint_list(conn, challenge_id).await {
        Ok(hints) => Ok(Json(hints)),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "Challenge not found")),
        Err(e) => {
            error!("Failed to get hint list: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get hint list"))
        }
    }
}

async fn delete_hint(
    State(ref conn): State<DatabaseConnection>, Query(query): Query<HintIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let hint_id = query.hint_id;
    match hint::delete_hint(conn, hint_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(DbErr::RecordNotFound(_)) => Err((StatusCode::NOT_FOUND, "Hint not found")),
        Err(e) => {
            error!("Failed to delete hint: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete hint"))
        }
    }
}
