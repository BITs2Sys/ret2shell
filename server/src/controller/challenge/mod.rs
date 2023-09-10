use super::{
    layer::{auth, info},
    GlobalState,
};
use crate::entity::{
    challenge, game,
    user::{self, Permission},
};
use crate::utility::string::deunicode_str;
use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

mod answer;
mod env;
mod hint;
mod repo;
mod submission;
mod traffic;
mod workflow;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_challenge))
        .route_layer(middleware::from_fn(auth::permission_required!(
            Permission::Organize
        )))
        .route("/", get(get_challenge_list))
        .route_layer(middleware::from_fn(
            auth::game_challenges_privilege_required,
        ))
        .route_layer(middleware::from_fn(auth::game_privilege_required))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_user_full_info,
        ))
        .route_layer(middleware::from_fn(auth::permission_required!(
            Permission::Verified
        )))
        .nest("/answer", answer::router(state))
        .nest(
            "/:challenge_id",
            Router::new()
                .route("/", patch(update_challenge).delete(delete_challenge))
                .route(
                    "/build",
                    get(get_challenge_build_status).post(add_challenge_to_build_queue),
                )
                .route_layer(middleware::from_fn(auth::permission_required!(
                    Permission::Organize
                )))
                .route("/", get(get_challenge_info))
                .route("/status", get(get_challenge_status))
                .route("/static", get(get_challenge_static_info))
                .route("/attachment", get(download_challenge_attachment))
                .route_layer(middleware::from_fn(auth::challenge_privilege_required))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    info::prepare_challenge_info,
                ))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    info::prepare_user_full_info,
                ))
                .route_layer(middleware::from_fn(auth::permission_required!(
                    Permission::Verified
                )))
                .nest("/submission", submission::router(state))
                .nest("/env", env::router(state))
                .nest("/repo", repo::router(state))
                .nest("/hint", hint::router(state)),
        )
}

async fn create_challenge(
    State(config): State<GlobalState>,
    State(ref conn): State<DatabaseConnection>,
    Json(mut challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // TODO: there may exists a same name challenge in the same game
    // Should valid?
    challenge.bucket = Some(format!(
        "{}/{}_{}",
        config.config.bucket.path,
        challenge.game_id,
        deunicode_str(&challenge.name)
    ));
    let created_challenge = match challenge::create_challenge(conn, challenge).await {
        Ok(created_challenge) => created_challenge,
        Err(err) => {
            error!("create_new_challenge error: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create new challenge",
            ));
        }
    };
    // TODO: add checker
    // new_checker(&created_challenge).await.map_err(|err| {
    //     error!("create_new_challenge error: {}", err);
    //     (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         "failed to create new challenge",
    //     )
    // })?;

    Ok(Json(created_challenge))
}

async fn update_challenge(
    State(ref conn): State<DatabaseConnection>,
    Extension(current_challenge): Extension<challenge::Model>,
    Json(mut challenge_udpate): Json<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    challenge_udpate.id = current_challenge.id;
    match challenge::update_challenge(conn, current_challenge.id, challenge_udpate).await {
        Ok(updated_challenge) => Ok(Json(updated_challenge)),
        Err(err) => {
            error!("update_challenge error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update challenge",
            ))
        }
    }
}

#[derive(Deserialize)]
struct ChallengeListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    game_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct ChallengeList {
    challenges: Vec<challenge::Model>,
    total: u64,
}

async fn get_challenge_list(
    State(ref conn): State<DatabaseConnection>,
    Query(params): Query<ChallengeListQuery>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let current_game = match params.game_id {
        Some(game_id) => match game::get_game(conn, game_id).await {
            Ok(Some(game)) => game,
            Ok(None) => {
                return Err((StatusCode::NOT_FOUND, "game not found"));
            }
            Err(err) => {
                error!("failed to get game: {}", err);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get game"));
            }
        },
        None => {
            return Err((StatusCode::BAD_REQUEST, "game_id needed"));
        }
    };

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match challenge::get_challenge_page_by_game_and_user(
        conn,
        current_game,
        current_user,
        page,
        per_page,
    )
    .await
    {
        Ok((challenges, total)) => Ok(Json(ChallengeList { challenges, total })),
        Err(err) => {
            error!("get_challenge_list error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get challenge list",
            ))
        }
    }
}

async fn delete_challenge(
    State(ref conn): State<DatabaseConnection>,
    Path(challenge_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match challenge::delete_challenge(conn, challenge_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("failed to delete challenge: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete challenge",
            ))
        }
    }
}

async fn get_challenge_info(
    Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json(challenge))
}

async fn get_challenge_status() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}

async fn get_challenge_static_info() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}

async fn download_challenge_attachment() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}

async fn get_challenge_build_status() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}

async fn add_challenge_to_build_queue() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}
