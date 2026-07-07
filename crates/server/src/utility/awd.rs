//! BITs2CTF fork: AWD (Attack-and-Defense) orchestration.
//!
//! Each team gets its own machine (a k8s pod), all interconnected. Every round the
//! platform rotates a fresh flag into every machine and runs an SLA service check;
//! teams attack each other's machines and submit the stolen flag. Attack points are
//! awarded immediately (decayed by how many teams have been exploited); SLA + defense
//! points are awarded at each round boundary (defense = service up AND not exploited).
//!
//! Round accounting is idempotent + atomic: the per-round DB writes (finalizing the
//! previous round's SLA/defense scoring and recording the new round's flags) happen
//! inside one transaction that also advances `awd_state.last_round`, and finalization
//! is guarded by a per-row `finalized` flag, so a retried tick can never double-award
//! or wedge on the unique `(challenge, team, round)` index.

use std::{collections::BTreeMap, time::Duration};

use nanoid::nanoid;
use r2s_config::cluster::{AwdConfig, ChallengeEnv};
use r2s_cluster::CHALLENGE_NS;
use r2s_database::{
  awd_instance, awd_round, awd_state, awd_steal, challenge, extra, game,
  team::{self, TeamScoreHistory, TeamScoreHistoryKind},
};
use sea_orm::{ConnectionTrait, TransactionTrait};
use tracing::{info, warn};

use crate::traits::{GlobalState, ResponseError};

const LABEL_ALPHABET: [char; 36] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

fn sha256_hex(bytes: &[u8]) -> String {
  ring::digest::digest(&ring::digest::SHA256, bytes)
    .as_ref()
    .iter()
    .fold(String::new(), |mut acc, b| {
      acc.push_str(&format!("{b:02x}"));
      acc
    })
}

/// POSIX single-quote a value for safe interpolation into a `/bin/sh -c` command:
/// wrap in single quotes and escape embedded single quotes as `'\''`.
fn sh_squote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "'\\''"))
}

fn env_from(config: &AwdConfig) -> ChallengeEnv {
  ChallengeEnv {
    internet: config.internet,
    restricted: config.restricted,
    privileged: config.privileged,
    images: vec![config.image.clone()],
    pull_secret: config.pull_secret.clone(),
  }
}

/// Cosine decay of the attack reward by how many teams have been exploited, using
/// the challenge's score-rule decay window.
fn attack_value(config: &AwdConfig, challenge: &challenge::Model, exploited_teams: u64) -> i32 {
  let initial = config.attack_reward.max(0);
  let minimum = (initial / 3).max(1);
  let decay = challenge.score_rule.decay.max(1) as u64;
  if exploited_teams <= 1 {
    return initial;
  }
  if exploited_teams >= decay {
    return minimum;
  }
  let ratio = (exploited_teams as f64 - 1.0) / (decay as f64 - 1.0);
  let curve = (ratio * std::f64::consts::PI).cos();
  let normalized = (curve + 1.0) / 2.0;
  (minimum as f64 + (initial - minimum) as f64 * normalized).round() as i32
}

/// Resolve a reachable `host:port` for a team's machine from its exposed NodePort.
/// Returns `None` when the challenge has no exposed port or the NodePort isn't
/// assigned yet (the caller retries lazily on the next round tick).
async fn resolve_address(
  state: &GlobalState, config: &AwdConfig, pod_name: &str,
) -> Option<String> {
  config.image.port?;
  let host = state
    .config
    .server
    .as_ref()
    .map(|server| server.external_domain.clone())
    .filter(|host| !host.is_empty())?;
  let node_port = state
    .cluster
    .at(CHALLENGE_NS)
    .service_node_port(pod_name)
    .await
    .ok()
    .flatten()?;
  Some(format!("{host}:{node_port}"))
}

/// Best-effort: fill in any instance whose reachable address hasn't been resolved yet
/// (NodePorts are assigned asynchronously after the service is created).
async fn resolve_missing_addresses(state: &GlobalState, config: &AwdConfig, challenge_id: i64) {
  let db = &state.db.conn;
  let Ok(instances) = awd_instance::list_by_challenge(db, challenge_id).await else {
    return;
  };
  for instance in instances {
    if instance.address.is_some() {
      continue;
    }
    if let Some(address) = resolve_address(state, config, &instance.pod_name).await {
      let _ = awd_instance::update_state(db, instance.id, "running", Some(address)).await;
    }
  }
}

/// Provision one machine (pod) per team for an AWD challenge. Idempotent: teams that
/// already have an instance row are skipped, and a pod left over from a partially
/// failed prior run (deterministic name) is adopted instead of losing the team.
pub async fn provision(
  state: &GlobalState, game: &game::Model, challenge: &challenge::Model, config: &AwdConfig,
) -> Result<usize, ResponseError> {
  let db = &state.db.conn;
  let (teams, _) = team::get_page(db, game.id, 1, 100_000, None, None, None, None, true).await?;
  let env_config = env_from(config);
  let cluster = state.cluster.at(CHALLENGE_NS);
  let node_selector = state
    .config
    .cluster
    .as_ref()
    .and_then(|c| c.node_selector.clone());
  let mut created = 0;
  for team in teams {
    if awd_instance::get_for_team(db, challenge.id, team.id)
      .await?
      .is_some()
    {
      continue;
    }
    let deterministic = format!("ret2shell-awd-{}-{}", challenge.id, team.id);
    // Reserve the slot atomically before touching k8s: the unique (challenge, team)
    // index means a concurrent provision (or a re-run) loses this insert and is
    // skipped, so exactly one call proceeds to create the pod.
    let reservation = match awd_instance::create(
      db,
      awd_instance::Model {
        id: 0,
        created_at: chrono::Utc::now(),
        challenge_id: challenge.id,
        team_id: team.id,
        pod_name: deterministic.clone(),
        address: None,
        status: "pending".to_owned(),
      },
    )
    .await
    {
      Ok(instance) => instance,
      Err(_) => continue, // another provision reserved this team's slot first
    };
    let traffic = nanoid!(21, &LABEL_ALPHABET);
    let labels = BTreeMap::from([
      ("ret.sh.cn/awd".to_owned(), "true".to_owned()),
      ("ret.sh.cn/challenge".to_owned(), challenge.id.to_string()),
      ("ret.sh.cn/team".to_owned(), team.id.to_string()),
      ("ret.sh.cn/traffic".to_owned(), traffic.clone()),
      (
        "ret.sh.cn/internet".to_owned(),
        config.internet.to_string(),
      ),
    ]);
    let annotations = BTreeMap::from([
      ("ret.sh.cn/challenge".to_owned(), challenge.name.clone()),
      ("ret.sh.cn/game".to_owned(), game.name.clone()),
    ]);
    match cluster
      .create_awd_env(
        challenge.id,
        team.id,
        labels,
        annotations,
        std::collections::HashMap::new(),
        env_config.clone(),
        node_selector.clone(),
        config.image.port.is_some(),
      )
      .await
    {
      Ok(snapshot) => {
        let pod_name = snapshot
          .pod
          .metadata
          .name
          .clone()
          .unwrap_or_else(|| deterministic.clone());
        let address = resolve_address(state, config, &pod_name).await;
        awd_instance::update_state(db, reservation.id, "running", address)
          .await
          .ok();
        created += 1;
      }
      // pod creation failed: release the reservation + clear any orphaned k8s object so
      // a re-run recreates cleanly.
      Err(err) => {
        cluster.delete_service(&deterministic).await.ok();
        cluster.delete_pod(&deterministic).await.ok();
        awd_instance::delete_by_id(db, reservation.id).await.ok();
        warn!(team = team.id, error = %err, "AWD provision failed; released reservation, will retry on next provision");
      }
    }
  }
  info!(challenge_id = challenge.id, created, "AWD machines provisioned");
  Ok(created)
}

/// Tear down all machines for an AWD challenge and clear its round/scoring state so a
/// later re-provision starts clean (stale steals would otherwise pre-decay rewards and
/// stale round rows would validate old flags).
pub async fn teardown(state: &GlobalState, challenge_id: i64) -> Result<(), ResponseError> {
  let db = &state.db.conn;
  let cluster = state.cluster.at(CHALLENGE_NS);
  for instance in awd_instance::list_by_challenge(db, challenge_id).await? {
    cluster.delete_service(&instance.pod_name).await.ok();
    cluster.delete_pod(&instance.pod_name).await.ok();
  }
  awd_instance::delete_by_challenge(db, challenge_id).await?;
  awd_round::delete_by_challenge(db, challenge_id).await?;
  awd_steal::delete_by_challenge(db, challenge_id).await?;
  awd_state::delete(db, challenge_id).await?;
  Ok(())
}

/// One round tick. Two phases: (A) rotate a fresh flag into every machine and run its
/// SLA check via pod exec (network I/O, done outside any DB transaction), then (B) in
/// a single transaction, finalize the previous round's SLA/defense scoring, record the
/// new round's flags idempotently, and advance `awd_state.last_round` atomically.
pub async fn round_tick(
  state: &GlobalState, _game: &game::Model, challenge: &challenge::Model, config: &AwdConfig,
) -> Result<(), ResponseError> {
  let db = &state.db.conn;
  let round_secs = config.round_secs.max(1) as i64;
  let now = chrono::Utc::now();
  let current_round = now.timestamp() / round_secs;
  let prev = awd_state::get(db, challenge.id)
    .await?
    .unwrap_or_else(|| awd_state::empty(challenge.id));
  if current_round <= prev.last_round && prev.last_round != 0 {
    // still within the current round; only heal any missing addresses.
    resolve_missing_addresses(state, config, challenge.id).await;
    return Ok(());
  }

  // --- Phase A: rotate flags + SLA check (pod exec, outside the DB transaction) ---
  // Injection necessarily precedes the Phase-B commit (SLA depends on the inject
  // result). If Phase B then rolls back on a transient DB error, a machine can briefly
  // hold a flag whose hash wasn't recorded; this self-heals on the very next tick
  // (~SCAN_INTERVAL_SECS), which re-injects and commits, so the exposure is a bounded
  // rejection window, never a permanent inconsistency.
  let cluster = state.cluster.at(CHALLENGE_NS);
  let timeout = Duration::from_secs(config.timeout_secs.max(1));
  let mut rotations: Vec<(i64, String, bool)> = Vec::new(); // (team_id, value_hash, sla_ok)
  let mut errors: Vec<String> = Vec::new();
  for instance in awd_instance::list_by_challenge(db, challenge.id).await? {
    let flag = format!("flag{{{}}}", nanoid!(32, &LABEL_ALPHABET));
    let hash = sha256_hex(flag.as_bytes());
    let inject = cluster
      .exec_pod(
        &instance.pod_name,
        None,
        vec![
          "/bin/sh".to_owned(),
          "-c".to_owned(),
          format!(
            "printf '%s' {} > {}",
            sh_squote(&flag),
            sh_squote(&config.flag_path)
          ),
        ],
        None,
        timeout,
      )
      .await;
    let injected = inject.map(|o| o.success).unwrap_or(false);
    if !injected {
      errors.push(format!("team {}: flag injection failed", instance.team_id));
    }
    let sla_ok = if let Some(cmd) = &config.check_command {
      injected
        && cluster
          .exec_pod(&instance.pod_name, None, cmd.clone(), None, timeout)
          .await
          .map(|o| o.success)
          .unwrap_or(false)
    } else {
      injected
    };
    rotations.push((instance.team_id, hash, sla_ok));
  }

  // --- Phase B: persist atomically (finalize prev round + record new round + advance) ---
  let txn = db.begin().await?;
  let locked = awd_state::get_for_update(&txn, challenge.id)
    .await?
    .unwrap_or_else(|| awd_state::empty(challenge.id));
  if current_round <= locked.last_round && locked.last_round != 0 {
    txn.rollback().await.ok();
    resolve_missing_addresses(state, config, challenge.id).await;
    return Ok(());
  }

  // finalize the just-completed round (SLA + defense = up and not exploited), guarded
  // by the per-row `finalized` flag so a retry never re-awards.
  if locked.last_round > 0 {
    for row in awd_round::list_unfinalized_by_round(&txn, challenge.id, locked.last_round).await? {
      if row.sla_ok {
        award(
          &txn,
          challenge,
          row.team_id,
          config.sla_reward,
          format!("AWD SLA up round {}", locked.last_round),
          now,
        )
        .await?;
        if !victim_attacked(&txn, challenge.id, locked.last_round, row.team_id).await? {
          award(
            &txn,
            challenge,
            row.team_id,
            config.defense_reward,
            format!("AWD defense round {}", locked.last_round),
            now,
          )
          .await?;
        }
      }
      awd_round::mark_finalized(&txn, row.id).await?;
    }
  }

  // record the new round's flags idempotently (upsert on the unique key).
  for (team_id, hash, sla_ok) in &rotations {
    awd_round::upsert(&txn, challenge.id, *team_id, current_round, hash, *sla_ok, now).await?;
  }
  awd_state::put(
    &txn,
    awd_state::Model {
      challenge_id: challenge.id,
      last_round: current_round,
      last_checked_at: Some(now),
      last_error: if errors.is_empty() {
        None
      } else {
        Some(errors.join("; "))
      },
    },
  )
  .await?;
  txn.commit().await?;

  resolve_missing_addresses(state, config, challenge.id).await;
  Ok(())
}

async fn victim_attacked<C>(
  db: &C, challenge_id: i64, round: i64, victim: i64,
) -> Result<bool, ResponseError>
where
  C: ConnectionTrait, {
  use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
  Ok(
    awd_steal::Entity::find()
      .filter(awd_steal::Column::ChallengeId.eq(challenge_id))
      .filter(awd_steal::Column::Round.eq(round))
      .filter(awd_steal::Column::VictimTeamId.eq(victim))
      .one(db)
      .await?
      .is_some(),
  )
}

/// Verify an AWD attack submission: does the flag match a *different* team's currently
/// live round flag? The authoritative round is the one the worker actually rotated
/// (`awd_state.last_round`), which advances in lock-step with the physical flag
/// injection — so it never runs ahead of the flag on the machine the way a wall-clock
/// round would. Only that round is accepted, so a flag from an already-finalized round
/// can't be replayed for extra credit.
pub async fn verify_attack<C>(
  db: &C, challenge: &challenge::Model, config: &AwdConfig, attacker_team_id: i64, submitted: &str,
) -> Result<(bool, String), ResponseError>
where
  C: ConnectionTrait, {
  let hash = sha256_hex(submitted.trim().as_bytes());
  // Lock awd_state so this attack serializes against the round worker's finalize +
  // advance (round_tick Phase B), preventing a stale read from crediting an attack
  // against a round whose defense was already scored.
  let state_round = match awd_state::get_for_update(db, challenge.id).await? {
    Some(state) if state.last_round > 0 => state.last_round,
    // worker hasn't recorded a round yet: fall back to wall-clock.
    _ => chrono::Utc::now().timestamp() / config.round_secs.max(1) as i64,
  };
  let Some(victim) = awd_round::find_by_hash(db, challenge.id, state_round, &hash).await? else {
    return Ok((false, "not a current-round flag".to_owned()));
  };
  if victim.finalized {
    return Ok((false, "this round has already been scored".to_owned()));
  }
  let round = victim.round;
  if victim.team_id == attacker_team_id {
    return Ok((false, "cannot submit your own flag".to_owned()));
  }
  if awd_steal::exists(db, challenge.id, round, attacker_team_id, victim.team_id).await? {
    return Ok((true, "already stolen this round".to_owned()));
  }
  let exploited = awd_steal::distinct_victim_count(db, challenge.id).await?;
  // Count this victim toward the decay only if it is newly exploited — a victim already
  // in the distinct set would otherwise be double-counted by the `+ 1`.
  let count = if awd_steal::victim_exploited(db, challenge.id, victim.team_id).await? {
    exploited
  } else {
    exploited + 1
  };
  let value = attack_value(config, challenge, count);
  let now = chrono::Utc::now();
  let extra = extra::create(
    db,
    extra::Model {
      id: 0,
      created_at: now,
      reason: format!(
        "AWD attack: stole team {}'s flag (round {round})",
        victim.team_id
      ),
      score: value,
      hint_id: None,
      team_id: attacker_team_id,
      challenge_id: Some(challenge.id),
    },
  )
  .await?;
  awd_steal::create(
    db,
    awd_steal::Model {
      id: 0,
      created_at: now,
      challenge_id: challenge.id,
      round,
      attacker_team_id,
      victim_team_id: victim.team_id,
      score: value,
      extra_id: extra.id,
    },
  )
  .await?;
  refresh_team_score(db, attacker_team_id, challenge.id, now).await?;
  Ok((true, format!("attack accepted (+{value})")))
}

async fn award<C>(
  db: &C, challenge: &challenge::Model, team_id: i64, score: i32, reason: String,
  now: chrono::DateTime<chrono::Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  if score <= 0 {
    return Ok(());
  }
  extra::create(
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
  refresh_team_score(db, team_id, challenge.id, now).await
}

async fn refresh_team_score<C>(
  db: &C, team_id: i64, challenge_id: i64, now: chrono::DateTime<chrono::Utc>,
) -> Result<(), ResponseError>
where
  C: ConnectionTrait, {
  let Some(mut team) = team::get_for_update(db, team_id).await? else {
    return Ok(());
  };
  let total = team::calc_score(db, team.id).await?;
  team.score = total;
  team.last_active_at = now;
  team.history.0.push(TeamScoreHistory {
    score: total,
    changed_at: now,
    challenge_id: Some(challenge_id),
    blood_state: None,
    kind: TeamScoreHistoryKind::Extra,
  });
  team::update(db, team).await?;
  Ok(())
}
