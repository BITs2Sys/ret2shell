//! BITs2CTF fork: per-challenge AWD (`awd.toml`) config + lifecycle + scoreboard.

use axum::{
  Extension, Json, Router,
  extract::State,
  response::IntoResponse,
  routing::{get, patch, post},
};
use chrono::Utc;
use r2s_bucket::Bucket;
use r2s_config::cluster::AwdConfig;
use r2s_database::{awd_instance, awd_state, awd_steal, challenge, game, team, user::Permission};
use serde::Serialize;

use crate::{
  middleware::{
    auth::{Token, is_game_admin},
    data::extract_team,
  },
  traits::{GlobalState, ResponseError},
  utility::awd,
};

pub(crate) fn admin_router() -> Router<GlobalState> {
  Router::new()
    .route("/awd", patch(update_awd_config).delete(delete_awd_config))
    .route("/awd/provision", post(provision_awd))
    .route("/awd/teardown", post(teardown_awd))
}

pub(crate) fn player_router() -> Router<GlobalState> {
  Router::new()
    .route("/awd", get(get_awd_status))
    .route("/awd/scoreboard", get(get_awd_scoreboard))
}

pub(crate) fn validate_awd_config(config: &AwdConfig) -> Result<(), ResponseError> {
  if !config.enabled {
    return Ok(());
  }
  if config.round_secs < 1 {
    return Err(ResponseError::BadRequest(
      "AWD round length must be greater than zero".to_owned(),
    ));
  }
  if config.image.name.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "AWD machine image is required".to_owned(),
    ));
  }
  // flag_path is interpolated into the in-pod flag-injection command; require an
  // absolute path free of control characters (it is also shell-quoted at use).
  let flag_path = config.flag_path.trim();
  if flag_path.is_empty() || !flag_path.starts_with('/') {
    return Err(ResponseError::BadRequest(
      "AWD flag path must be an absolute path (e.g. /flag)".to_owned(),
    ));
  }
  if flag_path.contains(['\n', '\r', '\0']) {
    return Err(ResponseError::BadRequest(
      "AWD flag path must not contain newlines or null bytes".to_owned(),
    ));
  }
  Ok(())
}

#[derive(Serialize)]
struct AwdStatus {
  config: Option<AwdConfig>,
  state: Option<awd_state::Model>,
  /// the requesting team's own machine, if any.
  instance: Option<awd_instance::Model>,
  round: i64,
}

async fn get_awd_status(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  team_ext: Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = super::get_challenge_bucket(&state.bucket, &game, &challenge).await?;
  let config = challenge_bucket.awd().await?;
  let admin = is_game_admin!(token, game);
  let round_secs = config
    .as_ref()
    .map(|c| c.round_secs.max(1))
    .unwrap_or(1) as i64;
  let round = Utc::now().timestamp() / round_secs;
  let team = extract_team!(game, team_ext, token);
  let instance = if let Some(team) = &team {
    awd_instance::get_for_team(&state.db.conn, challenge.id, team.id).await?
  } else {
    None
  };
  // internal per-challenge operational detail (last_error) is admin-only.
  let awd_state = awd_state::get(&state.db.conn, challenge.id).await?;
  let awd_state = if admin {
    awd_state
  } else {
    awd_state.map(|s| awd_state::Model {
      last_error: None,
      ..s
    })
  };
  Ok(Json(AwdStatus {
    config: config.map(|c| if admin { c } else { c.desensitize() }),
    state: awd_state,
    instance,
    round,
  }))
}

async fn get_awd_scoreboard(
  State(state): State<GlobalState>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    awd_steal::get_list_ex(&state.db.conn, challenge.id).await?,
  ))
}

async fn update_awd_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(config): Json<AwdConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  validate_awd_config(&config)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  challenge_bucket
    .set_awd(serde_json::to_value(&config)?)
    .await?;
  game_bucket
    .commit(
      format!(":crossed_swords: update AWD config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(config))
}

async fn delete_awd_config(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(&state.bucket, &game, &challenge).await?;
  challenge_bucket.delete_awd().await?;
  game_bucket
    .commit(
      format!(":fire: delete AWD config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

/// Admin: provision one machine per team for this AWD challenge.
async fn provision_awd(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  let challenge_bucket = super::get_challenge_bucket(&state.bucket, &game, &challenge).await?;
  let config = challenge_bucket.awd().await?.ok_or_else(|| {
    ResponseError::PreconditionFailed("this challenge is not an AWD challenge".to_owned())
  })?;
  let created = awd::provision(&state, &game, &challenge, &config).await?;
  Ok(Json(serde_json::json!({ "created": created })))
}

/// Admin: tear down all machines for this AWD challenge.
async fn teardown_awd(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  // Teardown wipes awd_steal (the decay counter) but leaves earned score; allowing it
  // mid-game would let teams re-farm at full value after a re-provision. Require the
  // game be hidden/not-in-progress first.
  if game.in_progress() {
    return Err(ResponseError::PreconditionFailed(
      "cannot tear down AWD machines while the game is in progress; hide the game first"
        .to_owned(),
    ));
  }
  awd::teardown(&state, challenge.id).await?;
  Ok(axum::http::StatusCode::NO_CONTENT)
}
