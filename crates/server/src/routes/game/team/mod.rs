use axum::{
  extract::{Query, State},
  middleware,
  response::IntoResponse,
  routing::{get, post},
  Extension, Json, Router,
};
use chrono::Utc;
use nanoid::nanoid;
use r2s_auditor::Auditor;
use r2s_database::{extra, game, team, user::Permission, user2_team};
use r2s_migrator::Database;
use serde::Deserialize;

use crate::{
  middleware::{
    auth::{self, Token},
    data,
  },
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/self", get(get_self_team))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      data::prepare_team_info,
    ))
    .route("/", post(create_team))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Verified
    )))
    .route("/", get(get_team_list))
    .nest(
      "/:team",
      Router::new()
        .route("/", get(get_team_info))
        .route("/extra", get(get_team_extra))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_data!(team, false),
        )),
    )
}

async fn get_self_team(
  Extension(team): Extension<team::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(team))
}

#[derive(Deserialize)]
struct TeamInfoQuery {
  pub ex: Option<bool>,
}

async fn get_team_info(
  State(ref db): State<Database>, Extension(team): Extension<team::Model>,
  Query(query): Query<TeamInfoQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  if query.ex.unwrap_or(false) {
    Ok(Json(team.into()))
  } else {
    Ok(Json(
      team::get_ex(&db.conn, team.id)
        .await?
        .ok_or(ResponseError::NotFound("team".to_string()))?,
    ))
  }
}

#[derive(Deserialize)]
struct TeamListQuery {
  pub filter: Option<String>,
  pub institute_id: Option<i64>,
  pub order_by: Option<String>,
  pub asc: Option<bool>,
  pub page: Option<u64>,
  pub page_size: Option<u64>,
  pub min_state: Option<team::State>,
}

async fn get_team_list(
  State(ref db): State<Database>, Extension(game): Extension<game::Model>,
  Extension(token): Extension<Token>, Query(query): Query<TeamListQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let min_state = if query
    .min_state
    .clone()
    .is_some_and(|s| s < team::State::Hidden)
    && !(token.permissions.0.contains(&Permission::Game) && game.admins.0.contains(&token.id))
  {
    Some(team::State::Hidden)
  } else {
    query.min_state
  };
  let results = team::get_page(
    &db.conn,
    game.id,
    query.page.unwrap_or(1),
    query.page_size.unwrap_or(15),
    min_state,
    query.institute_id,
    query.filter,
    query.order_by,
    query.asc.unwrap_or(true),
  )
  .await?;
  Ok(Json(results))
}

async fn get_team_extra(
  State(ref db): State<Database>, Extension(team): Extension<team::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(extra::get_list_ex(&db.conn, team.id).await?))
}

#[derive(Deserialize)]
struct CreateTeamRequest {
  pub name: String,
}

async fn create_team(
  State(ref db): State<Database>, State(ref auditor): State<Auditor>,
  Extension(game): Extension<game::Model>, Extension(token): Extension<Token>,
  Json(req): Json<CreateTeamRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  if game.register_at > Utc::now() {
    return Err(ResponseError::PreconditionFailed(
      "game has not start yet".to_owned(),
    ));
  }
  if (game.start_at < Utc::now() && !game.can_register_after_started) || game.end_at < Utc::now() {
    return Err(ResponseError::PreconditionFailed(
      "too late to participate".to_owned(),
    ));
  }
  if team::get_by_user_id(&db.conn, game.id, token.id)
    .await?
    .is_some()
  {
    return Err(ResponseError::Conflict(
      "can not join multiple teams".to_owned(),
    ));
  }
  let state = if game.enable_audit {
    if auditor.audit_content(&req.name) {
      team::State::Pending
    } else {
      team::State::Passed
    }
  } else {
    team::State::Passed
  };
  let team_token = Some(nanoid!());
  let team = team::create(
    &db.conn,
    team::Model {
      name: req.name,
      game_id: game.id,
      state,
      token: team_token,
      ..Default::default()
    },
  )
  .await?;
  user2_team::user_join_team(&db.conn, token.id, team.id).await?;
  Ok(Json(team))
}
