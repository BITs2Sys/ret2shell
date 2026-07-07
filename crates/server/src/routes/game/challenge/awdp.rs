//! BITs2CTF fork: per-challenge AWDP (`awdp.toml`) config + status + scoreboard.

use axum::{
  Extension, Json, Router,
  extract::State,
  response::IntoResponse,
  routing::{get, patch},
};
use chrono::Utc;
use r2s_bucket::Bucket;
use r2s_config::cluster::{AwdpConfig, AwdpMode};
use r2s_database::{awdp_award, awdp_solve, awdp_state, challenge, game, team, user::Permission};
use serde::Serialize;

use crate::{
  middleware::{
    auth::{Token, is_game_admin},
    data::extract_team,
  },
  traits::{GlobalState, ResponseError},
};

pub(crate) fn admin_router() -> Router<GlobalState> {
  Router::new().route("/awdp", patch(update_awdp_config).delete(delete_awdp_config))
}

pub(crate) fn player_router() -> Router<GlobalState> {
  Router::new()
    .route("/awdp", get(get_awdp_status))
    .route("/awdp/scoreboard", get(get_awdp_scoreboard))
}

pub(crate) fn validate_awdp_config(config: &AwdpConfig) -> Result<(), ResponseError> {
  if !config.enabled {
    return Ok(());
  }
  if config.round_secs < 1 {
    return Err(ResponseError::BadRequest(
      "AWDP round length must be greater than zero".to_owned(),
    ));
  }
  Ok(())
}

#[derive(Serialize)]
struct AwdpStatus {
  config: Option<AwdpConfig>,
  state: Option<awdp_state::Model>,
  /// whether the requesting team has solved (and thus earns the per-round bonus).
  solved: bool,
  solved_round: Option<i64>,
  round: i64,
}

async fn get_awdp_status(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  team_ext: Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = super::get_challenge_bucket(&state.bucket, &game, &challenge).await?;
  let config = challenge_bucket.awdp().await?;
  let round_secs = config
    .as_ref()
    .map(|c| c.round_secs.max(1))
    .unwrap_or(1) as i64;
  let round = Utc::now().timestamp() / round_secs;

  let team = extract_team!(game, team_ext, token);
  let solve = if let Some(team) = &team {
    awdp_solve::get_for_team(&state.db.conn, challenge.id, team.id).await?
  } else {
    None
  };

  Ok(Json(AwdpStatus {
    config,
    state: awdp_state::get(&state.db.conn, challenge.id).await?,
    solved: solve.is_some(),
    solved_round: solve.map(|s| s.solved_round),
    round,
  }))
}

async fn get_awdp_scoreboard(
  State(state): State<GlobalState>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    awdp_award::get_list_ex(&state.db.conn, challenge.id).await?,
  ))
}

async fn update_awdp_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(config): Json<AwdpConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  validate_awdp_config(&config)?;
  // fix-mode AWDP needs a fix.toml to be solvable by fixing.
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  if config.enabled
    && config.mode == AwdpMode::Fix
    && !challenge_bucket
      .fix()
      .await?
      .is_some_and(|fix| fix.enabled)
  {
    return Err(ResponseError::PreconditionFailed(
      "AWDP fix mode requires an enabled fix.toml".to_owned(),
    ));
  }
  challenge_bucket
    .set_awdp(serde_json::to_value(&config)?)
    .await?;
  game_bucket
    .commit(
      format!(":crossed_swords: update AWDP config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(config))
}

async fn delete_awdp_config(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(&state.bucket, &game, &challenge).await?;
  challenge_bucket.delete_awdp().await?;
  game_bucket
    .commit(
      format!(":fire: delete AWDP config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}
