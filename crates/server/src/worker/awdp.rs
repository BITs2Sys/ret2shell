//! BITs2CTF fork: AWDP (Attack-and-Defense-Plus) worker.
//!
//! Jeopardy-style solve/fix with a **persistent per-round bonus**: a team that
//! solves (or fixes) an AWDP challenge in round R earns the challenge's current
//! (decayed) score every round from R until the game ends. Fully standalone — it
//! polls solved submissions to detect solves, so it needs no edits to the core
//! submission/fix workers. Per-round idempotency comes from the unique
//! `awdp_award(challenge, team, round)` index.

use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};
use r2s_bucket::{Bucket, challenge::ChallengeBucket};
use r2s_database::{
  awdp_award, awdp_solve, awdp_state, challenge, extra, game, submission,
  team::{self, TeamScoreHistory, TeamScoreHistoryKind},
};
use sea_orm::{ConnectionTrait, TransactionTrait};
use tracing::{error, info, warn};

use crate::traits::{GlobalState, ResponseError};

const SCAN_INTERVAL_SECS: u64 = 5;

pub async fn spawn(state: GlobalState) {
  info!(interval_secs = SCAN_INTERVAL_SECS, "AWDP worker started");
  let mut ticker = tokio::time::interval(Duration::from_secs(SCAN_INTERVAL_SECS));
  loop {
    ticker.tick().await;
    if let Err(err) = scan_once(&state).await {
      error!(error=?err, "AWDP worker scan failed");
    }
  }
}

async fn scan_once(state: &GlobalState) -> Result<(), ResponseError> {
  let games = game::get_list(&state.db.conn, None, None, None, None).await?;
  for game in games {
    if !game.in_progress() {
      continue;
    }
    let challenges = challenge::get_full_list(&state.db.conn, game.id).await?;
    for challenge in challenges {
      if let Err(err) = process(state, &game, &challenge).await {
        warn!(challenge_id = challenge.id, error=?err, "AWDP process failed");
      }
    }
  }
  Ok(())
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

async fn process(
  state: &GlobalState, game: &game::Model, challenge: &challenge::Model,
) -> Result<(), ResponseError> {
  let Ok(challenge_bucket) = get_challenge_bucket(&state.bucket, game, challenge).await else {
    return Ok(());
  };
  let Some(config) = challenge_bucket.awdp().await.ok().flatten() else {
    return Ok(());
  };
  if !config.enabled {
    return Ok(());
  }
  let db = &state.db.conn;
  let round_secs = config.round_secs.max(1) as i64;
  let now = Utc::now();
  let round = now.timestamp() / round_secs;

  // optional total-rounds cap (measured from game start).
  if config.total_rounds > 0 {
    let elapsed_rounds = now
      .signed_duration_since(game.start_at)
      .num_seconds()
      .max(0)
      / round_secs;
    if elapsed_rounds >= config.total_rounds as i64 {
      return Ok(());
    }
  }

  // 1. detect solves: record awdp_solve (with the round of the earliest solve) for
  //    every team that has a solved submission (covers both flag-solve and fix).
  let solved = submission::get_list(
    db,
    true,
    false,
    Some(game.id),
    Some(challenge.id),
    None,
    None,
    false,
  )
  .await?;
  let mut earliest: HashMap<i64, i64> = HashMap::new();
  for submission in &solved {
    if let Some(team_id) = submission.team_id {
      let ts = submission.created_at.timestamp();
      earliest
        .entry(team_id)
        .and_modify(|e| {
          if ts < *e {
            *e = ts;
          }
        })
        .or_insert(ts);
    }
  }
  for (team_id, ts) in &earliest {
    if awdp_solve::get_for_team(db, challenge.id, *team_id)
      .await?
      .is_none()
    {
      awdp_solve::record(db, challenge.id, *team_id, ts / round_secs).await?;
    }
  }

  // 2. award every team whose solve round has arrived — the current round PLUS any
  //    rounds missed since the worker last ran (crash / redeploy / per-challenge
  //    error) or missed because the solve was detected late, so the persistent
  //    per-round bonus is never silently dropped. The `awdp_award(challenge, team,
  //    round)` unique index keeps this idempotent.
  let per_round_value = challenge.score;
  if per_round_value > 0 {
    for solve in awdp_solve::list_by_challenge(db, challenge.id).await? {
      if solve.solved_round > round {
        continue;
      }
      // resume from this team's own award frontier (not a global one): a solve that
      // is detected after the wall-clock round advanced past it still back-fills its
      // owed rounds. `None` -> never awarded -> start at the solve round.
      let start = match awdp_award::max_round_for_team(db, challenge.id, solve.team_id).await? {
        Some(highest) => (highest + 1).max(solve.solved_round),
        None => solve.solved_round,
      };
      for r in start..=round {
        if awdp_award::get_by_round_team(db, challenge.id, r, solve.team_id)
          .await?
          .is_some()
        {
          continue;
        }
        let txn = db.begin().await?;
        if awdp_award::get_by_round_team(&txn, challenge.id, r, solve.team_id)
          .await?
          .is_none()
        {
          award_round(&txn, challenge, solve.team_id, r, per_round_value, now).await?;
        }
        txn.commit().await?;
      }
    }
  }

  awdp_state::put(
    db,
    awdp_state::Model {
      challenge_id: challenge.id,
      last_round: round,
      last_checked_at: Some(now),
      last_error: None,
    },
  )
  .await?;
  Ok(())
}

async fn award_round<C>(
  db: &C, challenge: &challenge::Model, team_id: i64, round: i64, score: i32, now: DateTime<Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  let extra = extra::create(
    db,
    extra::Model {
      id: 0,
      created_at: now,
      reason: format!(
        "AWDP round {round} bonus for challenge {}:{}",
        challenge.id, challenge.name
      ),
      score,
      hint_id: None,
      team_id,
      challenge_id: Some(challenge.id),
    },
  )
  .await?;
  awdp_award::create(
    db,
    awdp_award::Model {
      id: 0,
      created_at: now,
      challenge_id: challenge.id,
      team_id,
      round,
      score,
      extra_id: extra.id,
    },
  )
  .await?;
  let Some(mut team) = team::get(db, team_id).await? else {
    return Err(ResponseError::NotFound("AWDP team not found".to_owned()));
  };
  let total = team::calc_score(db, team.id).await?;
  team.score = total;
  team.last_active_at = now;
  team.history.0.push(TeamScoreHistory {
    score: total,
    changed_at: now,
    challenge_id: Some(challenge.id),
    blood_state: None,
    kind: TeamScoreHistoryKind::Extra,
  });
  team::update(db, team).await?;
  Ok(())
}
