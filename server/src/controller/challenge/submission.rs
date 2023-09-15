use crate::{
    controller::{
        layer::{auth, info},
        GlobalState,
    },
    entity::{
        challenge, game, submission,
        user::{self, Permission},
    },
};
use axum::{
    extract::{Query, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/", get(get_challenge_submission_list))
        .route_layer(middleware::from_fn(auth::permission_required_any!(
            Permission::Organize,
            Permission::Devops,
            Permission::Audit
        )))
        .route("/", post(submit_flag))
        .route_layer(middleware::from_fn_with_state(
            _state.clone(),
            info::prepare_challenge_info,
        ))
        .route_layer(middleware::from_fn_with_state(
            _state.clone(),
            info::prepare_user_full_info,
        ))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
            Permission::Verified
        )))
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
    State(ref conn): State<DatabaseConnection>,
    Extension(challenge): Extension<challenge::Model>,
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

async fn submit_flag(
    Extension(ref conn): Extension<DatabaseConnection>,
    Extension(game): Extension<game::Model>,
    Extension(user): Extension<user::Model>,
    Extension(challenge): Extension<challenge::Model>,
    Json(mut submission): Json<submission::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // TODO: impl checker
    let result = true;

    submission.solved = result;
    submission.challenge_id = challenge.id;
    submission.user_id = user.id;
    submission.with_score = user.permissions.0.iter().all(|p| {
        !matches!(
            p,
            Permission::Audit | Permission::Devops | Permission::Organize
        )
    }) && game.host_as_game
        && game.in_progress()
        && result;
    if let Err(err) = submission::create_submission(conn, submission).await {
        error!("create_submission error: {}", err);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to create submission",
        ));
    }
    // TODO: refactor this to message queue with workers
    // debug!("checking whether to update scoreboard..");
    // debug!("game: {:?}, in progress: {}", game, game.in_progress());
    // if game.host_as_game && game.in_progress() && result {
    //     debug!("challenge solved, updating scoreboard..");
    //     let conn = conn.clone();
    //     let current_update_time = chrono::Utc::now();
    //     tokio::spawn(async move {
    //         let resp = match challenge::calc_challenge_score(&conn, &game, &challenge).await {
    //             Ok(resp) => resp,
    //             Err(err) => {
    //                 error!("failed to calc challenge status: {}", err);
    //                 return;
    //             }
    //         };
    //         let should_update_affected_teams = challenge.current_score != resp;
    //         if should_update_affected_teams {
    //             challenge.current_score = resp;
    //             match challenge::update_challenge_current_score(&conn, &challenge).await {
    //                 Ok(_) => {}
    //                 Err(err) => {
    //                     error!("failed to update challenge status: {}", err);
    //                 }
    //             };
    //         }
    //         let current_team = match team::get_team_by_user_id(&conn, game.id, user.id).await {
    //             Ok(Some(team)) => team,
    //             Ok(None) => {
    //                 error!("failed to get current team: team not found");
    //                 return;
    //             }
    //             Err(err) => {
    //                 error!("failed to get current team: {}", err);
    //                 return;
    //             }
    //         };
    //         let score = match team::calc_team_score(&conn, &current_team, game.id).await {
    //             Ok(score) => score,
    //             Err(err) => {
    //                 error!("failed to calc current team score: {:?}", err);
    //                 return;
    //             }
    //         };
    //         let _ = team::update_team_history_and_active_time(
    //             &conn,
    //             current_update_time,
    //             &current_team,
    //             score,
    //         )
    //         .await;
    //         if should_update_affected_teams {
    //             let teams = team::get_affected_teams_by_challenge_id(&conn, challenge.id)
    //                 .await
    //                 .map_err(|err| {
    //                     error!("failed to get affected teams: {}", err);
    //                 })
    //                 .unwrap_or_default();
    //             for team in teams {
    //                 let score = match team::calc_team_score(&conn, &team, game.id).await {
    //                     Ok(score) => score,
    //                     Err(err) => {
    //                         error!("failed to calc team score: {:?}", err);
    //                         continue;
    //                     }
    //                 };
    //                 let _ =
    //                     team::update_team_history_only(&conn, current_update_time, &team, score)
    //                         .await;
    //             }
    //         }
    //         // TODO: impl event push
    //     });
    // }
    Ok(Json(result))
}
