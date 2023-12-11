use axum::{
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{
        answer, challenge, game,
        user::{self, Permission},
    },
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route(
            "/",
            post(create_challenge_answer)
                .patch(update_challenge_answer)
                .delete(delete_challenge_answer),
        )
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Publish,
            Permission::Organize
        )))
        .route("/", get(get_challenge_answer))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Verified
        )))
}

async fn get_challenge_answer(
    State(ref conn): State<DatabaseConnection>, Extension(game): Extension<game::Model>,
    Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if game.host_as_game && !game.end_and_archive() {
        return Err((StatusCode::FORBIDDEN, "answer can not be seen in this time"));
    }
    match answer::get_answer_by_challenge_id(conn, challenge.id).await {
        Ok(Some(answer)) => Ok(Json(answer)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Answer not found")),
        Err(err) => {
            error!("Failed to get answer: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to get answer"))
        }
    }
}

async fn create_challenge_answer(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(challenge): Extension<challenge::Model>, Json(data): Json<answer::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match answer::create_answer(conn, user.id, challenge.id, data).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("Failed to create answer: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create answer"))
        }
    }
}

async fn update_challenge_answer(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(challenge): Extension<challenge::Model>, Json(data): Json<answer::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match answer::update_answer(conn, user.id, challenge.id, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to update answer: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update answer"))
        }
    }
}

async fn delete_challenge_answer(
    State(ref conn): State<DatabaseConnection>, Path(challenge_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match answer::delete_answer_by_challenge_id(conn, challenge_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to delete answer: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete answer"))
        }
    }
}
