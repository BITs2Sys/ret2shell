use axum::{
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use hyper::StatusCode;
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    audit::{word_filter::check_text, Auditor},
    cache::manager::RedisPool,
    captcha::captcha_protected,
    controller::{
        layer::{auth, info},
        GlobalState,
    },
    entity::{
        config, game,
        team::{self, TeamScoreHistory, TeamScoreHistoryList},
        user::{self, Permission},
        user2_team,
    },
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/info", patch(update_team_info))
        .route("/audit", patch(change_team_audit))
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Audit,
            Permission::Organize,
            Permission::Devops
        )))
        .route("/self/rank", get(get_self_team_rank))
        .route("/self/members", get(get_self_teammates))
        .route("/self", get(get_self_team_info))
        .route("/", post(create_team).patch(join_team))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::game_participate_privilege_required,
        ))
        .route("/info", get(get_team_info))
        .route("/info/members", get(get_team_teammates))
        .route("/", get(get_team_list))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_game_info_in_path,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_user_full_info,
        ))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Basic,
            Permission::Verified
        )))
}

#[derive(Deserialize)]
struct TeamIDQuery {
    pub team_id: i64,
}

#[derive(Deserialize)]
struct ListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub filter: Option<String>,
    pub need_audit: Option<bool>,
}

#[derive(Serialize)]
struct TeamList {
    pub teams: Vec<team::Model>,
    pub total: u64,
}

async fn get_team_list(
    State(ref conn): State<DatabaseConnection>, Path(game_id): Path<i64>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let filter = params.filter;
    let need_audit = params.need_audit;
    if page < 1 || per_page < 1 {
        error!("Invalid page={} or per_page={}", page, per_page);
        return Err((StatusCode::BAD_REQUEST, "invalid parameters"));
    }
    match team::get_team_page_by_game_id(conn, game_id, page, per_page, filter, need_audit).await {
        Ok((teams, total)) => Ok(Json(TeamList { teams, total })),
        Err(err) => {
            error!("Failed to get team list: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team list"))
        }
    }
}

#[derive(Deserialize)]
struct CreateTeamRequest {
    pub name: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn create_team(
    State(ref conn): State<DatabaseConnection>, State(mut cache): State<RedisPool>,
    State(ref auditor): State<Auditor>, Extension(config): Extension<config::Model>,
    Extension(user): Extension<user::Model>, Extension(game): Extension<game::Model>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    captcha_protected!(
        &config.captcha,
        &mut cache,
        &req.captcha_id,
        &req.captcha_answer
    );

    if user
        .permissions
        .0
        .iter()
        .any(|p| matches!(p, Permission::Devops | Permission::Organize))
    {
        return Err((StatusCode::FORBIDDEN, "host can not create team"));
    }

    let team_name = req.name.trim().to_string();

    match team::get_team_by_game_id_and_name(conn, game.id, &team_name).await {
        Ok(Some(_)) => {
            return Err((StatusCode::BAD_REQUEST, "team name already exists"));
        }
        Ok(None) => {}
        Err(err) => {
            error!("failed to check team name: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to check team name",
            ));
        }
    }
    let state = match game.enable_team_audit {
        true => {
            if check_text(&auditor.word_filter, &team_name) {
                team::State::NeedAudit
            } else {
                team::State::Normal
            }
        }
        false => team::State::Normal,
    };
    let team = team::Model {
        name: team_name,
        game_id: game.id,
        state,
        institute_id: user.institute_id,
        history: TeamScoreHistoryList(vec![TeamScoreHistory {
            score: 0,
            challenge_id: None,
            blood_state: None,
            time: Utc::now(),
        }]),
        ..Default::default()
    };

    let team = match team::create_team(conn, team.clone()).await {
        Ok(model) => model,
        Err(DbErr::RecordNotFound(_)) => return Err((StatusCode::BAD_REQUEST, "game not found")),
        Err(err) => {
            error!("Failed to create team: {}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to create team"));
        }
    };
    Ok(Json(
        user2_team::link_team_user(conn, user.id, team.id)
            .await
            .map_err(|err| {
                error!("Failed to link team with user: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to link team with user",
                )
            })?,
    ))
}

#[derive(Deserialize)]
struct JoinTeamRequest {
    pub token: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn join_team(
    State(ref conn): State<DatabaseConnection>, State(mut cache): State<RedisPool>,
    Extension(config): Extension<config::Model>, Extension(user): Extension<user::Model>,
    Extension(game): Extension<game::Model>, Json(request): Json<JoinTeamRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let id = request.captcha_id;
    let answer = request.captcha_answer;
    captcha_protected!(config.captcha, &mut cache, &id, &answer);

    if user
        .permissions
        .0
        .iter()
        .any(|p| matches!(p, Permission::Devops | Permission::Organize))
    {
        return Err((StatusCode::FORBIDDEN, "host can not create team"));
    }

    let team = match team::get_team_by_token(conn, &request.token).await {
        Ok(Some(team)) => team,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team by token: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get team by token",
            ));
        }
    };
    if team.institute_id.is_some() && team.institute_id != user.institute_id {
        return Err((
            StatusCode::FORBIDDEN,
            "you can not join team from other institute",
        ));
    }
    if team.state == team::State::Banned {
        return Err((StatusCode::BAD_REQUEST, "this team is banned"));
    }
    let size = match user2_team::get_team_size(conn, team.id).await {
        Ok(size) => size,
        Err(err) => {
            error!("failed to get team size: {}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team size"));
        }
    };
    if size >= game.team_size_limit as u64 {
        return Err((StatusCode::BAD_REQUEST, "team is full"));
    }
    match user2_team::link_team_user(conn, user.id, team.id).await {
        Ok(_) => Ok(Json(team)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to link team with user",
        )),
    }
}

async fn get_self_team_info(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Path(game_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match team::get_team_by_user_id(conn, game_id, user.id).await {
        Ok(Some(team)) => Ok(Json(team)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team info: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team info"))
        }
    }
}

async fn get_self_teammates(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Path(game_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let team = match team::get_team_by_user_id(conn, game_id, user.id).await {
        Ok(Some(team)) => Ok(team),
        Ok(None) => Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team info: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team info"))
        }
    }?;
    team::get_team_members(conn, team.id)
        .await
        .map(Json)
        .map_err(|err| {
            error!("failed to get team members: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get team members",
            )
        })
}

#[derive(Serialize)]
struct TeamRank {
    pub rank: u64,
}

async fn get_self_team_rank(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Path(game_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match team::get_team_rank_by_user_id(conn, game_id, user.id).await {
        Ok(Some(rank)) => Ok(Json(TeamRank { rank })),
        Ok(None) => Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team rank: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team rank"))
        }
    }
}

async fn get_team_info(
    State(ref conn): State<DatabaseConnection>, Extension(op_user): Extension<user::Model>,
    Query(query): Query<TeamIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let team_id = query.team_id;
    match team::get_team(conn, team_id).await {
        Ok(Some(team)) => {
            if op_user.permissions.0.contains(&Permission::Devops)
                || (op_user.permissions.0.contains(&Permission::Organize)
                    && op_user.institute_id == team.institute_id)
            {
                Ok(Json(team))
            } else {
                Ok(Json(team.desensitize()))
            }
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team info: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team info"))
        }
    }
}

async fn get_team_teammates(
    State(ref conn): State<DatabaseConnection>, Query(query): Query<TeamIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let team_id = query.team_id;
    team::get_team_members(conn, team_id)
        .await
        .map(Json)
        .map_err(|err| {
            error!("failed to get team members: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get team members",
            )
        })
}

async fn update_team_info(
    State(ref conn): State<DatabaseConnection>, Query(query): Query<TeamIDQuery>,
    Json(data): Json<team::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let team_id = query.team_id;
    match team::update_team(conn, team_id, data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to update team: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update team"))
        }
    }
}

async fn change_team_audit(
    State(ref conn): State<DatabaseConnection>, Query(query): Query<TeamIDQuery>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let team_id = query.team_id;
    let mut current_team = match team::get_team(conn, team_id).await {
        Ok(Some(current_team)) => current_team,
        Ok(None) => return Err((StatusCode::NOT_FOUND, "team not found")),
        Err(err) => {
            error!("failed to get team info: {}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to get team info"));
        }
    };
    match current_team.state {
        team::State::NeedAudit => current_team.state = team::State::Normal,
        team::State::Normal => current_team.state = team::State::NeedAudit,
        _ => {}
    }
    match team::update_team(conn, team_id, current_team).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to update team: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update team"))
        }
    }
}
