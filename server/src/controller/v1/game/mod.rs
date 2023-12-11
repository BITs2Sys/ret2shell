use axum::{
    extract::{Path, Query, State},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{
    controller::{
        layer::{
            auth::{self, pass_admin_for_game, Token},
            info,
        },
        GlobalState,
    },
    entity::{
        game, submission, team as team_entity,
        user::{self, Permission},
    },
};

mod notification;
mod team;
mod writeup;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", post(create_game))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Organize
        )))
        .route("/", get(get_game_list))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_user_full_info,
        ))
        .nest(
            "/:game_id",
            Router::new()
                .route("/", patch(update_game))
                .route_layer(middleware::from_fn(auth::permission_required_all!(
                    Permission::Organize
                )))
                .route("/submission", get(get_game_submission_list))
                // NOTE: maybe we should return submissions for different permssions?
                // EDIT: no, the models are different.
                .route_layer(middleware::from_fn(auth::permission_required_any!(
                    Permission::Organize,
                    Permission::Devops,
                    Permission::Audit
                )))
                .route("/solved", get(get_team_solved))
                .route("/", get(get_game))
                .route("/scoreboard", get(get_scoreboard))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    info::prepare_user_full_info,
                ))
                .route_layer(middleware::from_fn(auth::permission_required_any!(
                    Permission::Verified,
                    Permission::Statistics
                )))
                .nest("/notification", notification::router(state))
                .nest("/team", team::router(state))
                .nest("/writeup", writeup::router(state)),
        )
}

async fn create_game(
    State(ref conn): State<DatabaseConnection>, Json(game): Json<game::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match game::create_game(conn, game).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => {
            error!("Failed to create game: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to create game"))
        }
    }
}

#[derive(Deserialize)]
struct GameListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub host_as_game: Option<bool>,
}

#[derive(Serialize)]
struct GameList {
    pub games: Vec<game::Model>,
    pub total: u64,
}

async fn get_game_list(
    State(ref conn): State<DatabaseConnection>, op_user: Option<Extension<user::Model>>,
    Query(query): Query<GameListQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    let is_organizer = if let Some(Extension(op_user)) = op_user.as_ref() {
        op_user.permissions.0.contains(&Permission::Organize)
    } else {
        false
    };
    match game::get_game_page(conn, page, per_page, query.host_as_game, !is_organizer).await {
        Ok((games, total)) => Ok(Json(GameList { games, total })),
        Err(err) => {
            error!("Failed to get game list: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get game list"))
        }
    }
}

async fn get_game(
    State(ref conn): State<DatabaseConnection>, Extension(op_user): Extension<user::Model>,
    Path(game_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match game::get_game(conn, game_id).await {
        Ok(Some(model)) => {
            if model.hidden && !op_user.permissions.0.contains(&Permission::Organize) {
                warn!(
                    "user {}:{} tried to get hidden game {}:{}",
                    op_user.id, op_user.name, model.id, model.name
                );
                return Err((StatusCode::NOT_FOUND, "game not found"));
            }
            Ok(Json(model))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "game not found")),
        Err(err) => {
            error!("failed to get game model: {err}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get game info"))
        }
    }
}

async fn update_game(
    State(ref conn): State<DatabaseConnection>, Path(game_id): Path<i64>,
    Json(game): Json<game::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match game::update_game(conn, game_id, game).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("update game failed: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "update game failed"))
        }
    }
}

#[derive(Deserialize)]
struct SubmissionListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SubmissionList {
    pub submissions: Vec<submission::ModelWithInfo>,
    pub total: u64,
}

async fn get_game_submission_list(
    State(ref conn): State<DatabaseConnection>, Path(game_id): Path<i64>,
    Query(params): Query<SubmissionListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let (submissions, total) =
        submission::get_submission_page(conn, None, Some(game_id), page, per_page)
            .await
            .map_err(|err| {
                error!("failed to get submission list: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to get submission list",
                )
            })?;
    Ok(Json(SubmissionList { submissions, total }))
}

#[derive(Deserialize)]
struct ScoreboardListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub all: Option<bool>,
    pub institute: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct ScoreboardList {
    pub teams: Vec<team_entity::Model>,
    pub total: u64,
}

async fn get_scoreboard(
    State(ref conn): State<DatabaseConnection>, Path(game_id): Path<i64>,
    Query(params): Query<ScoreboardListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match team_entity::get_scoreboard(conn, game_id, page, per_page, params.all, params.institute)
        .await
    {
        Ok((teams, total)) => Ok(Json(ScoreboardList { teams, total })),
        Err(err) => {
            error!("failed to get scoreboard: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get scoreboard",
            ))
        }
    }
}

#[derive(Deserialize)]
pub struct TeamSolvedQuery {
    pub team_id: Option<i64>,
}

pub async fn get_team_solved(
    State(ref conn): State<DatabaseConnection>, Extension(token): Extension<Token>,
    Path(game_id): Path<i64>, Query(query): Query<TeamSolvedQuery>,
) -> Result<Response, (StatusCode, &'static str)> {
    let result = match query.team_id {
        Some(id) => {
            let mut result = submission::get_solved_submission_of_team(conn, game_id, id)
                .await
                .map_err(|err| {
                    error!("failed to get team solved challenges: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to get team solved challenges",
                    )
                })?;
            let admin = pass_admin_for_game!(token.permissions.0);
            if !admin {
                result = result
                    .into_iter()
                    .map(|r| submission::ModelWithInfo {
                        content: String::new(),
                        ..r
                    })
                    .collect::<Vec<_>>()
            }
            Json(result)
        }
        .into_response(),
        None => Json(
            submission::get_solved_submission_of_user(conn, game_id, token.id)
                .await
                .map_err(|err| {
                    error!("failed to get self solved challenges: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to get self solved challenges",
                    )
                })?,
        )
        .into_response(),
    };
    Ok(result)
}
