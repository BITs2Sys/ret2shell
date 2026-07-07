use std::{collections::HashSet, time::Duration};

use axum::{
  body::{Body, to_bytes},
  http::Request,
};
use chrono::Utc;
use r2s_bucket::{Bucket, challenge::ChallengeBucket};
use r2s_config::cluster::{KohConfig, KohMode};
use r2s_database::{
  challenge, extra, game, koh_award, koh_event, koh_identifier, koh_state,
  team::{self, TeamScoreHistory, TeamScoreHistoryKind},
};
use r2s_migrator::Database;
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::{
  traits::{GlobalState, ResponseError},
  utility::koh,
};

const KOH_SCAN_INTERVAL_SECS: u64 = 5;
const KOH_AGENT_BODY_LIMIT: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
struct AgentData {
  identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentResponse {
  success: Option<bool>,
  data: Option<AgentData>,
  identifier: Option<String>,
  message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoundRankEntryResponse {
  identifier: String,
  rank: Option<i32>,
  score: Option<i64>,
  metric: Option<i64>,
  message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RoundRankData {
  round: Option<i64>,
  tick: Option<i64>,
  rankings: Option<Vec<RoundRankEntryResponse>>,
  identifiers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct RoundRankResponse {
  success: Option<bool>,
  data: Option<RoundRankData>,
  round: Option<i64>,
  tick: Option<i64>,
  rankings: Option<Vec<RoundRankEntryResponse>>,
  identifiers: Option<Vec<String>>,
  message: Option<String>,
}

#[derive(Debug)]
struct RoundRankEntry {
  identifier: String,
  rank: i32,
  metric: Option<i64>,
  message: Option<String>,
}

#[derive(Debug)]
struct RoundRankCheck {
  round: Option<i64>,
  rankings: Vec<RoundRankEntry>,
}

pub async fn spawn(state: GlobalState) {
  info!(interval_secs = KOH_SCAN_INTERVAL_SECS, "KoH worker started");
  let mut ticker = tokio::time::interval(Duration::from_secs(KOH_SCAN_INTERVAL_SECS));
  loop {
    ticker.tick().await;
    if let Err(err) = scan_once(&state).await {
      error!(error=?err, "KoH worker scan failed");
    }
  }
}

async fn scan_once(state: &GlobalState) -> Result<(), ResponseError> {
  let games = game::get_list(&state.db.conn, None, None, None, None).await?;
  for game in games {
    let challenges = challenge::get_full_list(&state.db.conn, game.id).await?;
    for challenge in challenges {
      if let Err(err) = check_once(state, &game, &challenge, false).await {
        warn!(
          game_id = game.id,
          challenge_id = challenge.id,
          error=?err,
          "KoH challenge check failed"
        );
      }
    }
  }
  Ok(())
}

pub async fn check_once(
  state: &GlobalState, game: &game::Model, challenge: &challenge::Model, force: bool,
) -> Result<(), ResponseError> {
  let challenge_bucket = get_challenge_bucket(&state.bucket, game, challenge).await?;
  let Some(config) = challenge_bucket.koh().await? else {
    return Ok(());
  };
  if !config.enabled {
    koh::stop_hill_env(&state.cluster, challenge).await.ok();
    return Ok(());
  }
  if !is_active(game, challenge) {
    koh::stop_hill_env(&state.cluster, challenge).await.ok();
    return Ok(());
  }

  let interval_secs = match config.mode {
    KohMode::RoundRankHttp => config.round_secs.max(1),
    _ => config.interval_secs.max(1),
  };
  let now = Utc::now();
  let state_model = koh_state::get(&state.db.conn, challenge.id)
    .await?
    .unwrap_or_else(|| koh_state::empty(challenge.id));
  if !force
    && state_model
      .last_checked_at
      .is_some_and(|last| now.signed_duration_since(last).num_seconds() < interval_secs as i64)
  {
    return Ok(());
  }
  // KoH AgentHttp scoring is deliberately SAMPLE-BASED: each interval we check who holds
  // the hill *now* and award that single tick. Ticks are not back-filled if a scan is
  // delayed — the holder during an unsampled tick is unknown, so crediting the current
  // holder for it would be wrong. The (challenge, tick) award stays idempotent.
  let tick = now.timestamp() / interval_secs as i64;
  let env_config = challenge_bucket.env().await?;
  if config.auto_start
    && let Some(env_config) = env_config.clone()
  {
    if env_config.images.is_empty() || env_config.images.iter().all(|image| image.port.is_none()) {
      record_error(
        &state.db,
        challenge,
        "invalid_env",
        Some("KoH hill requires at least one service port".to_owned()),
        None,
      )
      .await?;
      return Ok(());
    }
    if let Some(snapshot) = koh::ensure_hill_env(
      &state.cluster,
      &state.config,
      &config,
      game,
      challenge,
      env_config,
    )
    .await?
    {
      debug!(pod=?snapshot.pod.metadata.name, "started KoH shared hill");
    }
  }

  let status_url =
    resolve_status_url(&state.cluster, challenge, &config, env_config.as_ref()).await?;
  match config.mode {
    KohMode::AgentHttp => {
      let checked = check_agent(state, &config, &status_url).await;
      match checked {
        Ok(identifier) => {
          process_identifier(&state.db, challenge, &config, identifier, tick).await?;
        }
        Err(message) => {
          record_error(&state.db, challenge, "error", Some(message), None).await?;
        }
      }
    }
    KohMode::RoundRankHttp => {
      let checked = check_round_rank_agent(state, &config, &status_url).await;
      match checked {
        Ok(result) => {
          process_rankings(&state.db, challenge, &config, result, tick).await?;
        }
        Err(message) => {
          record_error(&state.db, challenge, "error", Some(message), None).await?;
        }
      }
    }
    KohMode::GameElo => {
      record_error(
        &state.db,
        challenge,
        "unsupported_mode",
        Some("KoH game_elo mode is reserved for a future rating worker".to_owned()),
        Some(tick),
      )
      .await?;
    }
  }
  Ok(())
}

async fn get_challenge_bucket(
  bucket: &Bucket, game: &game::Model, challenge: &challenge::Model,
) -> Result<ChallengeBucket, ResponseError> {
  Ok(
    bucket
      .at(
        game
          .bucket
          .clone()
          .ok_or(ResponseError::PreconditionFailed(format!(
            "game {}:{} does not have a valid bucket",
            game.id, game.name
          )))?,
      )
      .await?
      .at(
        challenge
          .bucket
          .clone()
          .ok_or(ResponseError::PreconditionFailed(format!(
            "challenge {}:{} in game {}:{} does not have a valid bucket",
            challenge.id, challenge.name, game.id, game.name
          )))?,
      )
      .await?,
  )
}

fn is_active(game: &game::Model, challenge: &challenge::Model) -> bool {
  let now = Utc::now();
  game.in_progress()
    && !game.offline
    && !game.frozen
    && !challenge.hidden
    && challenge
      .release_at
      .is_none_or(|release_at| release_at <= now)
    && challenge
      .archive_at
      .is_none_or(|archive_at| archive_at > now)
}

async fn resolve_status_url(
  cluster: &r2s_cluster::Cluster, challenge: &challenge::Model, config: &KohConfig,
  env_config: Option<&r2s_config::cluster::ChallengeEnv>,
) -> Result<String, ResponseError> {
  if let Some(url) = config
    .status_url
    .clone()
    .filter(|url| !url.trim().is_empty())
  {
    return Ok(url);
  }
  let env_config = env_config.ok_or(ResponseError::PreconditionFailed(
    "KoH requires status_url or an online environment".to_owned(),
  ))?;
  let port = koh::infer_agent_port(config, env_config).ok_or(ResponseError::PreconditionFailed(
    "KoH agent port is not configured and cannot be inferred".to_owned(),
  ))?;
  let pods = cluster
    .at(r2s_cluster::CHALLENGE_NS)
    .get_koh_hill_env(challenge.id)
    .await?;
  let pod = pods.first().ok_or(ResponseError::PreconditionFailed(
    "KoH hill is not running yet".to_owned(),
  ))?;
  let pod_name = pod
    .metadata
    .name
    .clone()
    .ok_or(ResponseError::PreconditionFailed(
      "KoH hill pod has no name".to_owned(),
    ))?;
  Ok(koh::internal_status_url(
    &pod_name,
    port,
    &config.status_path,
  ))
}

async fn check_agent(
  state: &GlobalState, config: &KohConfig, status_url: &str,
) -> Result<Option<String>, String> {
  let mut builder = Request::builder().method("GET").uri(status_url);
  if let Some(api_key) = config.api_key.clone().filter(|key| !key.is_empty()) {
    builder = builder.header("x-api-key", api_key);
  }
  let request = builder
    .body(Body::empty())
    .map_err(|err| format!("failed to build KoH agent request: {err}"))?;
  let response = tokio::time::timeout(
    Duration::from_secs(config.timeout_secs.max(1)),
    state.requestor.request(request),
  )
  .await
  .map_err(|_| "KoH agent request timed out".to_owned())?
  .map_err(|err| format!("KoH agent request failed: {err}"))?;
  if !response.status().is_success() {
    return Err(format!("KoH agent returned {}", response.status()));
  }
  let bytes = to_bytes(Body::new(response.into_body()), KOH_AGENT_BODY_LIMIT)
    .await
    .map_err(|err| format!("failed to read KoH agent response: {err}"))?;
  let response = serde_json::from_slice::<AgentResponse>(&bytes)
    .map_err(|err| format!("failed to parse KoH agent response: {err}"))?;
  if response.success == Some(false) {
    return Err(
      response
        .message
        .unwrap_or_else(|| "KoH agent returned success=false".to_owned()),
    );
  }
  let identifier = response
    .data
    .and_then(|data| data.identifier)
    .or(response.identifier)
    .map(|identifier| identifier.trim().to_owned())
    .filter(|identifier| !identifier.is_empty());
  Ok(identifier)
}

async fn check_round_rank_agent(
  state: &GlobalState, config: &KohConfig, status_url: &str,
) -> Result<RoundRankCheck, String> {
  let mut builder = Request::builder().method("GET").uri(status_url);
  if let Some(api_key) = config.api_key.clone().filter(|key| !key.is_empty()) {
    builder = builder.header("x-api-key", api_key);
  }
  let request = builder
    .body(Body::empty())
    .map_err(|err| format!("failed to build KoH agent request: {err}"))?;
  let response = tokio::time::timeout(
    Duration::from_secs(config.timeout_secs.max(1)),
    state.requestor.request(request),
  )
  .await
  .map_err(|_| "KoH agent request timed out".to_owned())?
  .map_err(|err| format!("KoH agent request failed: {err}"))?;
  if !response.status().is_success() {
    return Err(format!("KoH agent returned {}", response.status()));
  }
  let bytes = to_bytes(Body::new(response.into_body()), KOH_AGENT_BODY_LIMIT)
    .await
    .map_err(|err| format!("failed to read KoH agent response: {err}"))?;
  let response = serde_json::from_slice::<RoundRankResponse>(&bytes)
    .map_err(|err| format!("failed to parse KoH rank response: {err}"))?;
  if response.success == Some(false) {
    return Err(
      response
        .message
        .unwrap_or_else(|| "KoH agent returned success=false".to_owned()),
    );
  }
  let mut round = response.round.or(response.tick);
  let mut rankings = response.rankings;
  let mut identifiers = response.identifiers;
  if let Some(data) = response.data {
    round = data.round.or(data.tick).or(round);
    rankings = data.rankings.or(rankings);
    identifiers = data.identifiers.or(identifiers);
  }
  let rankings = rankings.unwrap_or_else(|| {
    identifiers
      .unwrap_or_default()
      .into_iter()
      .map(|identifier| RoundRankEntryResponse {
        identifier,
        rank: None,
        score: None,
        metric: None,
        message: None,
      })
      .collect()
  });
  let rankings = rankings
    .into_iter()
    .enumerate()
    .filter_map(|(index, entry)| {
      let identifier = entry.identifier.trim().to_owned();
      (!identifier.is_empty()).then_some(RoundRankEntry {
        identifier,
        rank: entry.rank.unwrap_or(index as i32 + 1),
        metric: entry.metric.or(entry.score),
        message: entry.message,
      })
    })
    .collect();
  Ok(RoundRankCheck { round, rankings })
}

// BITs2CTF fork: centralized KoH scoring helpers (A4). Every award flows through
// `award_points`, so the extra + koh_award + team-score/history sequence lives in
// exactly one place and cannot drift between the identifier and ranking paths.

/// Recompute a team's score from its extras and push a KoH history entry.
async fn refresh_team_score<C>(
  db: &C, team_id: i64, challenge_id: i64, now: chrono::DateTime<Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  let Some(mut team) = team::get_for_update(db, team_id).await? else {
    return Err(ResponseError::NotFound("KoH team not found".to_owned()));
  };
  let score = team::calc_score(db, team.id).await?;
  team.score = score;
  team.last_active_at = now;
  team.history.0.push(TeamScoreHistory {
    score,
    changed_at: now,
    challenge_id: Some(challenge_id),
    blood_state: None,
    kind: TeamScoreHistoryKind::Koh,
  });
  team::update(db, team).await?;
  Ok(())
}

/// Create an `extra` + `koh_award` for one team and refresh its score.
#[allow(clippy::too_many_arguments)]
async fn award_points<C>(
  db: &C, challenge: &challenge::Model, team_id: i64, tick: i64, rank: Option<i32>,
  percent: Option<i32>, score: i32, reason: String, now: chrono::DateTime<Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  let extra = extra::create(
    db,
    extra::Model {
      id: 0,
      created_at: now,
      reason,
      score,
      hint_id: None,
      team_id,
      challenge_id: Some(challenge.id),
    },
  )
  .await?;
  koh_award::create(
    db,
    koh_award::Model {
      id: 0,
      created_at: now,
      challenge_id: challenge.id,
      team_id,
      tick,
      rank,
      percent,
      score,
      extra_id: extra.id,
    },
  )
  .await?;
  refresh_team_score(db, team_id, challenge.id, now).await
}

/// Reverse a prior award: delete its `extra` (which cascade-deletes the
/// `koh_award`) and recompute the affected team's score.
async fn revoke_award<C>(
  db: &C, award: &koh_award::Model, challenge_id: i64, now: chrono::DateTime<Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  extra::delete(db, award.extra_id).await?;
  refresh_team_score(db, award.team_id, challenge_id, now).await
}

async fn process_identifier(
  db: &Database, challenge: &challenge::Model, config: &KohConfig, identifier: Option<String>,
  tick: i64,
) -> Result<(), ResponseError> {
  let now = Utc::now();
  let txn = db.conn.begin().await?;
  let previous_state = koh_state::get_for_update(&txn, challenge.id)
    .await?
    .unwrap_or_else(|| koh_state::empty(challenge.id));
  let previous_team_id = previous_state.current_team_id;
  let mut state = previous_state.clone();
  state.last_checked_at = Some(now);
  state.current_identifier = identifier.clone();
  state.last_error = None;

  let mut status = "empty".to_owned();
  let mut message = None;
  let mut score_delta = 0;
  let mut team_id = None;

  if let Some(identifier) = identifier.as_deref() {
    match koh_identifier::get_by_identifier(&txn, challenge.id, identifier).await? {
      Some(owner) => {
        team_id = Some(owner.team_id);
        state.current_team_id = Some(owner.team_id);
        // BITs2CTF fork (B1): the holder at tick-end gets that tick's score, one
        // award per tick. If a different team was credited earlier in this tick
        // (only reachable via a forced re-check), re-point the award to the
        // current holder instead of blocking them.
        let existing = koh_award::get_by_tick(&txn, challenge.id, tick).await?;
        if config.reward <= 0 {
          status = "held".to_owned();
        } else {
          match existing {
            Some(award) if award.team_id == owner.team_id => {
              status = "held".to_owned();
            }
            Some(award) => {
              revoke_award(&txn, &award, challenge.id, now).await?;
              award_points(
                &txn,
                challenge,
                owner.team_id,
                tick,
                None,
                None,
                config.reward,
                format!(
                  "KoH hold reward for challenge {}:{} at tick {}",
                  challenge.id, challenge.name, tick
                ),
                now,
              )
              .await?;
              state.last_awarded_at = Some(now);
              status = "captured".to_owned();
              score_delta = config.reward;
            }
            None => {
              award_points(
                &txn,
                challenge,
                owner.team_id,
                tick,
                None,
                None,
                config.reward,
                format!(
                  "KoH hold reward for challenge {}:{} at tick {}",
                  challenge.id, challenge.name, tick
                ),
                now,
              )
              .await?;
              state.last_awarded_at = Some(now);
              status = if previous_team_id != Some(owner.team_id) {
                "captured".to_owned()
              } else {
                "awarded".to_owned()
              };
              score_delta = config.reward;
            }
          }
        }
      }
      None => {
        state.current_team_id = None;
        state.last_error = Some("unknown identifier".to_owned());
        status = "unknown_identifier".to_owned();
        message = Some("identifier does not belong to any team".to_owned());
      }
    }
  } else {
    state.current_team_id = None;
  }

  koh_event::create(
    &txn,
    koh_event::Model {
      id: 0,
      created_at: now,
      challenge_id: challenge.id,
      team_id,
      previous_team_id,
      identifier,
      status,
      message,
      score_delta,
      tick: Some(tick),
    },
  )
  .await?;
  koh_state::put(&txn, state).await?;
  txn.commit().await?;
  Ok(())
}

async fn process_rankings(
  db: &Database, challenge: &challenge::Model, config: &KohConfig, result: RoundRankCheck,
  fallback_tick: i64,
) -> Result<(), ResponseError> {
  let now = Utc::now();
  let round = result.round.unwrap_or(fallback_tick);
  let txn = db.conn.begin().await?;
  let previous_state = koh_state::get_for_update(&txn, challenge.id)
    .await?
    .unwrap_or_else(|| koh_state::empty(challenge.id));
  let previous_team_id = previous_state.current_team_id;
  let mut state = previous_state.clone();
  state.last_checked_at = Some(now);
  state.last_error = None;

  if round <= 0 {
    koh_event::create(
      &txn,
      koh_event::Model {
        id: 0,
        created_at: now,
        challenge_id: challenge.id,
        team_id: None,
        previous_team_id,
        identifier: None,
        status: "pending_round".to_owned(),
        message: Some("no completed KoH round is available yet".to_owned()),
        score_delta: 0,
        tick: Some(round),
      },
    )
    .await?;
    koh_state::put(&txn, state).await?;
    txn.commit().await?;
    return Ok(());
  }
  if config.total_rounds > 0 && round > config.total_rounds as i64 {
    koh_event::create(
      &txn,
      koh_event::Model {
        id: 0,
        created_at: now,
        challenge_id: challenge.id,
        team_id: None,
        previous_team_id,
        identifier: state.current_identifier.clone(),
        status: "completed".to_owned(),
        message: Some(format!("round {round} is beyond configured total rounds")),
        score_delta: 0,
        tick: Some(round),
      },
    )
    .await?;
    koh_state::put(&txn, state).await?;
    txn.commit().await?;
    return Ok(());
  }

  let mut rankings = result.rankings;
  rankings.sort_by_key(|entry| entry.rank);
  let first = rankings.first();
  state.current_identifier = first.map(|entry| entry.identifier.clone());
  state.current_team_id = match first {
    Some(entry) => koh_identifier::get_by_identifier(&txn, challenge.id, &entry.identifier)
      .await?
      .map(|identifier| identifier.team_id),
    None => None,
  };

  let rank_count = config.rank_count as usize;
  let mut seen_teams = HashSet::new();
  let mut awarded = 0;
  let mut total_delta = 0;
  let mut skipped = Vec::new();

  for entry in rankings {
    if entry.rank <= 0 || entry.rank as usize > rank_count {
      continue;
    }
    let percent = config
      .rank_percentages
      .get(entry.rank as usize - 1)
      .copied()
      .unwrap_or_default();
    if percent <= 0 || config.reward <= 0 {
      // BITs2CTF fork (B6): surface silent zero-score drops in the event log.
      skipped.push(format!(
        "rank {} scores 0 (percent {}%, reward {})",
        entry.rank, percent, config.reward
      ));
      continue;
    }
    let score = ((config.reward as i64 * percent as i64 + 50) / 100) as i32;
    if score <= 0 {
      skipped.push(format!(
        "rank {} rounds to 0 ({}% of reward {})",
        entry.rank, percent, config.reward
      ));
      continue;
    }
    let Some(owner) =
      koh_identifier::get_by_identifier(&txn, challenge.id, &entry.identifier).await?
    else {
      skipped.push(format!("unknown identifier {}", entry.identifier));
      continue;
    };
    if !seen_teams.insert(owner.team_id) {
      skipped.push(format!("duplicate team {}", owner.team_id));
      continue;
    }
    if koh_award::get_by_tick_team(&txn, challenge.id, round, owner.team_id)
      .await?
      .is_some()
    {
      skipped.push(format!(
        "team {} already awarded for round {}",
        owner.team_id, round
      ));
      continue;
    }

    award_points(
      &txn,
      challenge,
      owner.team_id,
      round,
      Some(entry.rank),
      Some(percent),
      score,
      format!(
        "KoH rank reward for challenge {}:{} round {} rank {} ({}%)",
        challenge.id, challenge.name, round, entry.rank, percent
      ),
      now,
    )
    .await?;
    koh_event::create(
      &txn,
      koh_event::Model {
        id: 0,
        created_at: now,
        challenge_id: challenge.id,
        team_id: Some(owner.team_id),
        previous_team_id,
        identifier: Some(entry.identifier.clone()),
        status: "rank_awarded".to_owned(),
        message: Some(
          entry
            .message
            .or_else(|| entry.metric.map(|metric| format!("metric={metric}")))
            .unwrap_or_else(|| format!("round {round} rank {}", entry.rank)),
        ),
        score_delta: score,
        tick: Some(round),
      },
    )
    .await?;
    state.last_awarded_at = Some(now);
    awarded += 1;
    total_delta += score;
  }

  if awarded == 0 {
    koh_event::create(
      &txn,
      koh_event::Model {
        id: 0,
        created_at: now,
        challenge_id: challenge.id,
        team_id: state.current_team_id,
        previous_team_id,
        identifier: state.current_identifier.clone(),
        status: if skipped.is_empty() {
          "empty".to_owned()
        } else {
          "rank_skipped".to_owned()
        },
        message: if skipped.is_empty() {
          Some(format!("round {round} has no ranked submissions"))
        } else {
          Some(skipped.join("; "))
        },
        score_delta: 0,
        tick: Some(round),
      },
    )
    .await?;
  } else {
    koh_event::create(
      &txn,
      koh_event::Model {
        id: 0,
        created_at: now,
        challenge_id: challenge.id,
        team_id: state.current_team_id,
        previous_team_id,
        identifier: state.current_identifier.clone(),
        status: "round_awarded".to_owned(),
        message: Some(format!("round {round}: awarded {awarded} team(s)")),
        score_delta: total_delta,
        tick: Some(round),
      },
    )
    .await?;
  }

  koh_state::put(&txn, state).await?;
  txn.commit().await?;
  Ok(())
}

async fn record_error(
  db: &Database, challenge: &challenge::Model, status: &str, message: Option<String>,
  tick: Option<i64>,
) -> Result<(), ResponseError> {
  let now = Utc::now();
  let txn = db.conn.begin().await?;
  let mut state = koh_state::get(&txn, challenge.id)
    .await?
    .unwrap_or_else(|| koh_state::empty(challenge.id));
  state.last_checked_at = Some(now);
  state.last_error = message.clone();
  koh_event::create(
    &txn,
    koh_event::Model {
      id: 0,
      created_at: now,
      challenge_id: challenge.id,
      team_id: None,
      previous_team_id: state.current_team_id,
      identifier: state.current_identifier.clone(),
      status: status.to_owned(),
      message,
      score_delta: 0,
      tick,
    },
  )
  .await?;
  koh_state::put(&txn, state).await?;
  txn.commit().await?;
  Ok(())
}
