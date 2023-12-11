// use crate::utility::string::deunicode_str;
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

use crate::{
    controller::{
        layer::{auth, info},
        GlobalState,
    },
    entity::{
        challenge, game, instance, submission as submission_entity,
        user::{self, Permission},
    },
};

mod tag;
// for challenge answer
mod answer;
// for challenge env
mod env;
// user hint, organizers publish
mod hint;
// for challenge storage, devops only
mod repo;
// user submit, organizers audit
mod submission;
// for traffic audit, organizers only
mod traffic;
// for build workflow, devops only
mod workflow;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_challenge))
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Organize,
            Permission::Devops
        )))
        .route("/", get(get_challenge_list))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::game_player_privilege_required,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_game_info_in_query,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_user_full_info,
        ))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Verified
        )))
        .nest("/tag", tag::router(state))
        .nest(
            "/:challenge_id",
            Router::new()
                .route("/", patch(update_challenge).delete(delete_challenge))
                .route("/statistics", get(get_challenge_statistics))
                .nest("/workflow", workflow::router(state))
                .nest("/repo", repo::router(state))
                .route_layer(middleware::from_fn(auth::permission_required_any!(
                    Permission::Organize,
                    Permission::Devops
                )))
                .route("/", get(get_challenge_info))
                .route("/attachment", get(download_challenge_attachment))
                .nest("/env", env::router(state))
                .nest("/submission", submission::router(state))
                .route("/solved-team", get(get_solved_team_list))
                .route("/solved-user", get(get_solved_user_list))
                .nest("/hint", hint::router(state))
                .nest("/answer", answer::router(state))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::challenge_privilege_required,
                ))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    info::prepare_challenge_info,
                ))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    info::prepare_user_full_info,
                ))
                .route_layer(middleware::from_fn(auth::permission_required_all!(
                    Permission::Verified
                ))),
        )
}

#[derive(Deserialize)]
struct GameIDQuery {
    pub game_id: i64,
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
}

#[derive(Serialize)]
struct SolvedUserList {
    users: Vec<submission_entity::ModelOnlyUserInfo>,
    total: u64,
}

async fn get_solved_user_list(
    State(ref conn): State<DatabaseConnection>, Extension(challenge): Extension<challenge::Model>,
    Query(params): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid paginate parameters"));
    }
    match submission_entity::get_solved_user_page(conn, challenge.id, page, per_page).await {
        Ok((users, total)) => Ok(Json(SolvedUserList { users, total })),
        Err(err) => {
            error!("get_solved_user_list error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get solved user list due to database error",
            ))
        }
    }
}

#[derive(Serialize)]
struct SolvedTeamList {
    teams: Vec<submission_entity::ModelOnlyTeamInfo>,
    total: u64,
}

async fn get_solved_team_list(
    State(ref conn): State<DatabaseConnection>, Extension(challenge): Extension<challenge::Model>,
    Query(params): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid paginate parameters"));
    }
    match submission_entity::get_solved_team_page(conn, challenge.id, page, per_page).await {
        Ok((teams, total)) => Ok(Json(SolvedTeamList { teams, total })),
        Err(err) => {
            error!("get_solved_team_list error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get solved team list due to database error",
            ))
        }
    }
}

async fn create_challenge(
    // State(config): State<GlobalState>,
    State(ref conn): State<DatabaseConnection>,
    Query(game_query): Query<GameIDQuery>,
    Json(mut challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    challenge.game_id = game_query.game_id;
    let created_challenge = match challenge::create_challenge(conn, challenge).await {
        Ok(created_challenge) => created_challenge,
        Err(err) => {
            error!("failed to create new challenge in database: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create new challenge in database",
            ));
        }
    };
    // TODO: there may exists a same name challenge in the same game
    // Should valid?
    // created_challenge.bucket = Some(format!(
    //     "{}_{}",
    //     created_challenge.id,
    //     deunicode_str(&created_challenge.name)
    // ));
    // Bucket::initialize(config.config.bucket.path, &created_challenge)
    //     .await
    //     .map_err(|err| {
    //         error!("failed to init challenge bucket: {}", err);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             "failed to init challenge bucket",
    //         )
    //     })?;

    // challenge::update_challenge_bucket(conn, created_challenge.id,
    // &created_challenge)     .await
    //     .map_err(|err| {
    //         error!("failed to update challenge bucket: {}", err);
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             "failed to update challenge bucket",
    //         )
    //     })?;

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
                "failed to update challenge in database",
            ))
        }
    }
}

#[derive(Deserialize)]
struct ChallengeListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    game_id: Option<i64>,
    tag_id: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct ChallengeList {
    challenges: Vec<challenge::Model>,
    total: u64,
}

async fn get_challenge_list(
    State(ref conn): State<DatabaseConnection>, Extension(current_user): Extension<user::Model>,
    Query(params): Query<ChallengeListQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let current_game = match params.game_id {
        Some(game_id) => match game::get_game(conn, game_id).await {
            Ok(Some(game)) => game,
            Ok(None) => {
                return Err((StatusCode::NOT_FOUND, "game not found"));
            }
            Err(err) => {
                error!("failed to get game: {}", err);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to get game due to database error",
                ));
            }
        },
        None => {
            return Err((StatusCode::BAD_REQUEST, "game_id needed"));
        }
    };

    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let tag_id = params.tag_id;
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid paginate parameters"));
    }
    match challenge::get_challenge_page_by_game_and_user(
        conn,
        current_game,
        current_user,
        page,
        per_page,
        tag_id,
    )
    .await
    {
        Ok((challenges, total)) => Ok(Json(ChallengeList { challenges, total })),
        Err(err) => {
            error!("get_challenge_list error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get challenge list due to database error",
            ))
        }
    }
}

async fn delete_challenge(
    State(ref conn): State<DatabaseConnection>, Path(challenge_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // TODO: delete additional models like build actions, containers, etc.
    match challenge::delete_challenge(conn, challenge_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("failed to delete challenge: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete challenge due to database error",
            ))
        }
    }
}

async fn get_challenge_info(
    Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json(challenge))
}

async fn download_challenge_attachment() -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json("to be implemented"))
}

#[derive(Serialize)]
struct StatisticsResponse {
    pub submissions_count: u64,
    pub solves_count: u64,
    pub instances_count: u64,
    pub running_instances_count: u64,
}

async fn get_challenge_statistics(
    State(ref db): State<DatabaseConnection>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let submissions_count = submission_entity::count_submissions(db, challenge.id, None)
        .await
        .map_err(|err| {
            error!("failed to get submissions count: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get submissions count",
            )
        })?;
    let solves_count = submission_entity::count_submissions(db, challenge.id, Some(true))
        .await
        .map_err(|err| {
            error!("failed to get solves count: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get solves count",
            )
        })?;

    let instances_count = instance::count_challenge_instance(db, challenge.id)
        .await
        .map_err(|err| {
            error!("failed to get instances count: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get instances count",
            )
        })?;
    let running_instances_count = instance::count_challenge_running_instance(db, challenge.id)
        .await
        .map_err(|err| {
            error!("failed to get running instances count: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get running instances count",
            )
        })?;

    Ok(Json(StatisticsResponse {
        submissions_count,
        solves_count,
        instances_count,
        running_instances_count,
    }))
}
