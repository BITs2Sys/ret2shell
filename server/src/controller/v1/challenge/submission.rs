use axum::{
    extract::{Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    controller::{layer::auth, GlobalState},
    entity::{
        challenge, extra, game, submission,
        team::{self, TeamScoreHistory},
        user::{self, Permission},
    },
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", get(get_challenge_submission_list))
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Organize,
            Permission::Devops,
            Permission::Audit
        )))
        .route("/", post(submit_flag))
}

#[derive(Deserialize)]
struct ListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SubmissionList {
    pub submissions: Vec<submission::ModelWithInfo>,
    pub total: u64,
}

async fn get_challenge_submission_list(
    State(ref conn): State<DatabaseConnection>, Extension(challenge): Extension<challenge::Model>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    match submission::get_submission_page(conn, Some(challenge.id), None, page, per_page).await {
        Ok((submissions, total)) => Ok(Json(SubmissionList { submissions, total })),
        Err(err) => {
            error!("Failed to get submission list: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get submission list",
            ))
        }
    }
}

#[derive(Serialize)]
struct SubmitResult {
    pub result: bool,
    pub blood_state: i32,
}

async fn submit_flag(
    State(ref conn): State<DatabaseConnection>, Extension(user): Extension<user::Model>,
    Extension(mut challenge): Extension<challenge::Model>,
    Json(mut submission): Json<submission::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // TODO: impl checker
    let result = true;

    if user.permissions.0.iter().all(|p| {
        matches!(
            p,
            Permission::Audit | Permission::Devops | Permission::Organize
        )
    }) {
        return Ok(Json(SubmitResult {
            result,
            blood_state: 0,
        }));
    }

    let mut team: Option<team::Model> = None;

    let game = game::get_game(conn, challenge.game_id)
        .await
        .map_err(|err| {
            error!("get_game error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to get game")
        })?
        .ok_or((StatusCode::NOT_FOUND, "game not found"))?;
    submission.solved = result;
    submission.challenge_id = challenge.id;
    submission.user_id = user.id;
    submission.team_id = None;
    let mut with_score = false;
    if game.host_as_game && game.in_progress() && result {
        team = team::get_team_by_user_id(conn, game.id, user.id)
            .await
            .map_err(|err| {
                error!("get_team_by_user_id error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to get team")
            })?;
        if let Some(team) = team.clone() {
            submission.team_id = Some(team.id);
            with_score = true;
        }
    }
    if let Err(err) = submission::create_submission(conn, submission).await {
        error!("create_submission error: {}", err);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to create submission",
        ));
    }
    let conn = conn.clone();
    let mut blood_state: i32 = 0;

    if with_score {
        if let Some(team) = team {
            let updated_time = Utc::now();
            let challenge_current_score = challenge::calc_challenge_score(&conn, &game, &challenge)
                .await
                .map_err(|err| {
                    error!("calc_challenge_score error: {}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to calc challenge score",
                    )
                })?;
            blood_state = submission::count_solves(&conn, challenge.id)
                .await
                .map_err(|err| {
                    error!("count_blood error: {}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "failed to count blood")
                })? as i32;
            if blood_state > 0 && blood_state <= 3 {
                let award_score = ((challenge.initial_score as f64
                    * (game.blood_award_rate as f64 / 100.0))
                    / blood_state as f64)
                    .round() as i32;
                extra::create_extra(
                    &conn,
                    extra::Model {
                        id: 0,
                        created_at: updated_time,
                        team_id: team.id,
                        reason: format!(
                            "No.{} solution for challenge {}:<{}>",
                            blood_state, challenge.id, challenge.name
                        ),
                        score: award_score,
                        hint_id: None,
                        challenge_id: Some(challenge.id),
                    },
                )
                .await
                .map_err(|err| {
                    error!("create_extra error: {}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "failed to create award")
                })?;
            }
            challenge.current_score = challenge_current_score;
            challenge::update_challenge_current_score(&conn, &challenge)
                .await
                .map_err(|err| {
                    error!("update_challenge_current_score error: {}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to update challenge current score",
                    )
                })?;
            if let Err(err) = team::update_team_score(&conn, &team, game.id, updated_time).await {
                error!("failed to update team score: {}", err);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to update team score",
                ));
            }

            tokio::spawn(async move {
                if let Ok(affected_teams) =
                    team::get_affected_teams_by_challenge_id(&conn, challenge.id)
                        .await
                        .map_err(|err| {
                            error!("get_affected_teams_by_challenge_id error: {}", err);
                        })
                {
                    for mut affected_team in affected_teams {
                        if affected_team.id == team.id {
                            let score_history = TeamScoreHistory {
                                score: affected_team.score,
                                time: updated_time,
                                challenge_id: Some(challenge.id),
                                blood_state: Some(blood_state),
                            };
                            affected_team.history.0.push(score_history);
                        } else {
                            match team::update_team_score(&conn, &team, game.id, updated_time).await
                            {
                                Ok(score) => {
                                    let score_history = TeamScoreHistory {
                                        score,
                                        time: updated_time,
                                        challenge_id: None,
                                        blood_state: None,
                                    };
                                    affected_team.history.0.push(score_history);
                                }
                                Err(err) => {
                                    error!("failed to update team score: {}", err);
                                    continue;
                                }
                            }
                        }
                        team::update_team_history(&conn, &affected_team)
                            .await
                            .map_err(|err| {
                                error!("update_team_history error: {}", err);
                            })
                            .ok();
                    }
                }
            });
        }
    }

    // TODO push event to all connected clients

    Ok(Json(SubmitResult {
        result,
        blood_state,
    }))
}
