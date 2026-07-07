//! BITs2CTF fork: AWD round worker — drives the per-round flag rotation, SLA
//! checks, and defense/SLA scoring for every enabled AWD challenge.

use std::time::Duration;

use r2s_bucket::{Bucket, challenge::ChallengeBucket};
use r2s_database::{challenge, game};
use tracing::{error, info, warn};

use crate::{
  traits::{GlobalState, ResponseError},
  utility::awd,
};

const SCAN_INTERVAL_SECS: u64 = 10;

pub async fn spawn(state: GlobalState) {
  info!(interval_secs = SCAN_INTERVAL_SECS, "AWD worker started");
  let mut ticker = tokio::time::interval(Duration::from_secs(SCAN_INTERVAL_SECS));
  loop {
    ticker.tick().await;
    if let Err(err) = scan_once(&state).await {
      error!(error=?err, "AWD worker scan failed");
    }
  }
}

async fn get_challenge_bucket(
  bucket: &Bucket, game: &game::Model, challenge: &challenge::Model,
) -> Result<ChallengeBucket, ResponseError> {
  let gb = game
    .bucket
    .clone()
    .ok_or_else(|| ResponseError::PreconditionFailed(format!("game {} has no bucket", game.id)))?;
  let cb = challenge.bucket.clone().ok_or_else(|| {
    ResponseError::PreconditionFailed(format!("challenge {} has no bucket", challenge.id))
  })?;
  Ok(bucket.at(gb).await?.at(cb).await?)
}

async fn scan_once(state: &GlobalState) -> Result<(), ResponseError> {
  let games = game::get_list(&state.db.conn, None, None, None, None).await?;
  for game in games {
    if !game.in_progress() {
      continue;
    }
    for challenge in challenge::get_full_list(&state.db.conn, game.id).await? {
      let Ok(challenge_bucket) = get_challenge_bucket(&state.bucket, &game, &challenge).await else {
        continue;
      };
      let Some(config) = challenge_bucket.awd().await.ok().flatten() else {
        continue;
      };
      if !config.enabled {
        continue;
      }
      if let Err(err) = awd::round_tick(state, &game, &challenge, &config).await {
        warn!(challenge_id = challenge.id, error=?err, "AWD round tick failed");
      }
    }
  }
  Ok(())
}
