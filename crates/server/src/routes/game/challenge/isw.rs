//! BITs2CTF fork: per-challenge ISW (`isw.toml`) config routes, mirroring the
//! koh config CRUD. Binds a challenge's flag to a guest-VM injection target; the
//! range lifecycle (arm/reset) lives under the top-level `/range` admin routes.

use axum::{
  Extension, Json, Router,
  extract::State,
  response::IntoResponse,
  routing::{get, patch},
};
use r2s_bucket::Bucket;
use r2s_config::cluster::IswConfig;
use r2s_database::{challenge, game, user::Permission};

use crate::{
  middleware::auth::{Token, is_game_admin},
  traits::{GlobalState, ResponseError},
};

/// Admin route: edit/delete the `isw.toml` manifest (merged above the admin layer).
pub(crate) fn admin_router() -> Router<GlobalState> {
  Router::new().route("/isw", patch(update_isw_config).delete(delete_isw_config))
}

/// Player route: read the (desensitized) config (merged below the admin layer).
pub(crate) fn player_router() -> Router<GlobalState> {
  Router::new().route("/isw", get(get_isw_config))
}

pub(crate) fn validate_isw_config(config: &IswConfig) -> Result<(), ResponseError> {
  if !config.enabled {
    return Ok(());
  }
  if config.range_template.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "ISW range template is required".to_owned(),
    ));
  }
  if config.vm.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "ISW target VM is required".to_owned(),
    ));
  }
  if config.guest_path.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "ISW guest flag path is required".to_owned(),
    ));
  }
  // `mode` is a chmod-style octal string injected into the guest; reject anything
  // that isn't 3-4 octal digits early rather than failing at inject time.
  let mode = config.mode.trim();
  if !mode.is_empty()
    && (!(3..=4).contains(&mode.len()) || !mode.bytes().all(|b| (b'0'..=b'7').contains(&b)))
  {
    return Err(ResponseError::BadRequest(
      "ISW file mode must be a 3-4 digit octal string (e.g. 0644)".to_owned(),
    ));
  }
  Ok(())
}

async fn get_isw_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = super::get_challenge_bucket(bucket, &game, &challenge).await?;
  let config = challenge_bucket.isw().await?;
  let config = if is_game_admin!(token, game) {
    config
  } else {
    config.map(IswConfig::desensitize)
  };
  Ok(Json(config))
}

async fn update_isw_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(config): Json<IswConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  validate_isw_config(&config)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  challenge_bucket
    .set_isw(serde_json::to_value(&config)?)
    .await?;
  game_bucket
    .commit(
      format!(":shield: update ISW config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(config))
}

async fn delete_isw_config(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(&state.bucket, &game, &challenge).await?;
  challenge_bucket.delete_isw().await?;
  game_bucket
    .commit(
      format!(":fire: delete ISW config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}
