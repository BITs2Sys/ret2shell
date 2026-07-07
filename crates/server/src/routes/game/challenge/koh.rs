use std::collections::HashMap;

use axum::{
  Extension, Json, Router,
  extract::{Query, State},
  response::IntoResponse,
  routing::{get, post},
};
use chrono::{DateTime, Utc, serde::ts_seconds_option};
use nanoid::nanoid;
use r2s_bucket::Bucket;
use r2s_cache::Cache;
use r2s_cluster::{CHALLENGE_NS, Cluster, Pod, traffic::MappedPort};
use r2s_config::cluster::{KohConfig, KohMode};
use r2s_database::{
  challenge, game, koh_award, koh_event, koh_identifier, koh_state, team, user::Permission,
};
use r2s_engine::Engine;
use r2s_migrator::Database;
use serde::{Deserialize, Serialize};

use crate::{
  middleware::auth::{Token, is_game_admin},
  routes::game::get_pod_field,
  traits::{GlobalState, ResponseError},
  utility::koh as koh_util,
  worker,
};

const IDENTIFIER_ALPHABET: [char; 36] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Serialize)]
pub(super) struct KohTarget {
  pub state: String,
  pub name: String,
  pub traffic: String,
  pub ports: Vec<u16>,
  pub target_port: Option<u16>,
  pub exposed_ports: Option<Vec<MappedPort>>,
}

#[derive(Serialize)]
pub(super) struct KohStatus {
  pub config: Option<KohConfig>,
  pub state: Option<koh_state::Model>,
  pub identifier: Option<koh_identifier::Model>,
  pub target: Option<KohTarget>,
}

#[derive(Serialize)]
pub(super) struct KohScore {
  pub team_id: i64,
  pub team_name: Option<String>,
  pub score: i32,
  #[serde(with = "ts_seconds_option")]
  pub last_awarded_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub(super) struct EventQuery {
  pub limit: Option<u64>,
}

/// BITs2CTF fork (A5): KoH admin routes, merged above the `game_admin_required`
/// layer by the challenge router — keeps the fork's footprint in `mod.rs` to a
/// single `.merge()` call instead of several scattered `.route()`s.
pub(crate) fn admin_router() -> Router<GlobalState> {
  Router::new()
    .route("/koh/hill", post(start_koh_hill).delete(stop_koh_hill))
    .route("/koh/check", post(check_koh_once))
}

/// KoH player-facing routes, merged below the admin layer. Config edit is guarded
/// by the inner `is_game_admin!` check in the handlers.
pub(crate) fn player_router() -> Router<GlobalState> {
  Router::new()
    .route(
      "/koh",
      get(get_koh_status)
        .patch(update_koh_config)
        .delete(delete_koh_config),
    )
    .route("/koh/event", get(get_koh_events))
    .route("/koh/scoreboard", get(get_koh_scoreboard))
}

pub(crate) fn validate_koh_config(config: &KohConfig) -> Result<(), ResponseError> {
  if !config.enabled {
    return Ok(());
  }
  // BITs2CTF fork: GameElo is reserved for a future rating worker and awards
  // nothing today — reject it rather than let an admin configure a silent no-op.
  if config.mode == KohMode::GameElo {
    return Err(ResponseError::BadRequest(
      "KoH game_elo mode is not implemented yet".to_owned(),
    ));
  }
  if config.interval_secs < 1 {
    return Err(ResponseError::BadRequest(
      "KoH interval must be greater than zero".to_owned(),
    ));
  }
  if config.round_secs < 1 {
    return Err(ResponseError::BadRequest(
      "KoH round length must be greater than zero".to_owned(),
    ));
  }
  if config.reward < 0 {
    return Err(ResponseError::BadRequest(
      "KoH reward cannot be negative".to_owned(),
    ));
  }
  if config.mode == KohMode::RoundRankHttp {
    if config.rank_count < 1 {
      return Err(ResponseError::BadRequest(
        "KoH scored team count must be greater than zero".to_owned(),
      ));
    }
    if config.rank_percentages.len() < config.rank_count as usize {
      return Err(ResponseError::BadRequest(
        "KoH rank percentages must cover every scored team".to_owned(),
      ));
    }
    if config
      .rank_percentages
      .iter()
      .take(config.rank_count as usize)
      .any(|percentage| !(0..=100).contains(percentage))
    {
      return Err(ResponseError::BadRequest(
        "KoH rank percentages must be between 0 and 100".to_owned(),
      ));
    }
  }
  if config.timeout_secs < 1 {
    return Err(ResponseError::BadRequest(
      "KoH timeout must be greater than zero".to_owned(),
    ));
  }
  if config.mode == KohMode::AgentHttp && config.status_path.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "KoH status path cannot be empty".to_owned(),
    ));
  }
  Ok(())
}

async fn ensure_team_identifier(
  db: &Database, game: &game::Model, challenge: &challenge::Model, team: Option<&team::Model>,
) -> Result<Option<koh_identifier::Model>, ResponseError> {
  let Some(team) = team else {
    return Ok(None);
  };
  if team.game_id != game.id {
    return Ok(None);
  }
  if let Some(identifier) = koh_identifier::get_by_team(&db.conn, challenge.id, team.id).await? {
    return Ok(Some(identifier));
  }
  for _ in 0..5 {
    let identifier = format!("koh_{}", nanoid!(16, &IDENTIFIER_ALPHABET));
    if koh_identifier::get_by_identifier(&db.conn, challenge.id, &identifier)
      .await?
      .is_some()
    {
      continue;
    }
    return Ok(Some(
      koh_identifier::create(
        &db.conn,
        koh_identifier::Model {
          id: 0,
          created_at: Utc::now(),
          challenge_id: challenge.id,
          team_id: team.id,
          identifier,
        },
      )
      .await?,
    ));
  }
  Err(ResponseError::Conflict(
    "failed to allocate a unique KoH identifier".to_owned(),
  ))
}

async fn expose_target(
  cluster: &Cluster, cache: &Cache, engine: &Engine, config: &r2s_config::GlobalConfig,
  game: &game::Model, pod: Pod,
) -> Result<KohTarget, ResponseError> {
  let state = pod
    .status
    .as_ref()
    .and_then(|status| status.phase.clone())
    .unwrap_or_else(|| "Unknown".to_owned());
  let name = pod.metadata.name.clone().unwrap_or_default();
  let traffic = get_pod_field!(pod, labels, "ret.sh.cn/traffic");
  let ports = get_pod_field!(pod, annotations, "ret.sh.cn/ports")
    .split(',')
    .filter_map(|port| port.parse::<u16>().ok())
    .collect::<Vec<_>>();
  let target_port = ports.first().copied();
  let mut target = KohTarget {
    state,
    name: name.clone(),
    traffic: traffic.clone(),
    ports,
    target_port,
    exposed_ports: None,
  };
  let (_, traffic_script) = koh_util::traffic_script(game, config)?;
  let Some(traffic_script) = traffic_script.filter(|script| !script.is_empty()) else {
    return Ok(target);
  };
  if cache.at("traffic").exists(&traffic).await? {
    target.exposed_ports = cache.at("traffic").get(&traffic).await?;
    return Ok(target);
  }
  let service = cluster.at(CHALLENGE_NS).get_service(&name).await?;
  let traffic_mapper = cluster
    .traffic
    .clone()
    .ok_or(ResponseError::InternalServerError(
      "traffic mapper is not initialized".to_owned(),
    ))?;
  let (traffic_key, _) = koh_util::traffic_script(game, config)?;
  traffic_mapper
    .preload(engine, &traffic_key, &traffic_script)
    .await?;
  let exposed_ports = traffic_mapper
    .expose(engine, &traffic_key, pod, service)
    .await?;
  cache
    .at("traffic")
    .set_ex(&traffic, &exposed_ports, 3600)
    .await?;
  target.exposed_ports = Some(exposed_ports);
  Ok(target)
}

async fn get_target(
  cluster: &Cluster, cache: &Cache, engine: &Engine, config: &r2s_config::GlobalConfig,
  game: &game::Model, challenge: &challenge::Model,
) -> Result<Option<KohTarget>, ResponseError> {
  let pods = cluster
    .at(CHALLENGE_NS)
    .get_koh_hill_env(challenge.id)
    .await?;
  let Some(pod) = pods.into_iter().next() else {
    return Ok(None);
  };
  Ok(Some(
    expose_target(cluster, cache, engine, config, game, pod).await?,
  ))
}

pub(super) async fn get_koh_status(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  team_ext: Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = super::get_challenge_bucket(&state.bucket, &game, &challenge).await?;
  let config = challenge_bucket.koh().await?;
  let identifier = if config.as_ref().is_some_and(|config| config.enabled) {
    ensure_team_identifier(&state.db, &game, &challenge, team_ext.0.as_ref()).await?
  } else {
    None
  };
  let target = if config.as_ref().is_some_and(|config| config.enabled) {
    get_target(
      &state.cluster,
      &state.cache,
      &state.engine,
      &state.config,
      &game,
      &challenge,
    )
    .await?
  } else {
    None
  };
  let config = if is_game_admin!(token, game) {
    config
  } else {
    config.map(KohConfig::desensitize)
  };
  Ok(Json(KohStatus {
    config,
    state: koh_state::get(&state.db.conn, challenge.id).await?,
    identifier,
    target,
  }))
}

pub(super) async fn update_koh_config(
  State(ref bucket): State<Bucket>, State(db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(config): Json<KohConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  validate_koh_config(&config)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  // BITs2CTF fork (B5): the KoH tick namespace is derived from interval/round
  // length; changing it after scoring has started would corrupt the
  // (challenge, tick) award idempotency. Reject such changes once awards exist.
  if let Some(previous) = challenge_bucket.koh().await?
    && (previous.interval_secs != config.interval_secs
      || previous.round_secs != config.round_secs)
    && koh_award::exists_for_challenge(&db.conn, challenge.id).await?
  {
    return Err(ResponseError::PreconditionFailed(
      "cannot change KoH interval/round length after scoring has started".to_owned(),
    ));
  }
  challenge_bucket
    .set_koh(serde_json::to_value(&config)?)
    .await?;
  game_bucket
    .commit(
      format!(":crown: update KoH config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(config))
}

pub(super) async fn delete_koh_config(
  State(state): State<GlobalState>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !is_game_admin!(token, game) {
    return Err(ResponseError::Forbidden("permission denied".to_owned()));
  }
  super::check_challenge_publishing(&challenge)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(&state.bucket, &game, &challenge).await?;
  challenge_bucket.delete_koh().await?;
  koh_util::stop_hill_env(&state.cluster, &challenge)
    .await
    .ok();
  game_bucket
    .commit(
      format!(":fire: delete KoH config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

pub(super) async fn start_koh_hill(
  State(state): State<GlobalState>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = super::get_challenge_bucket(&state.bucket, &game, &challenge).await?;
  let config = challenge_bucket
    .koh()
    .await?
    .ok_or(ResponseError::PreconditionFailed(
      "KoH config is not set".to_owned(),
    ))?;
  if !config.enabled {
    return Err(ResponseError::PreconditionFailed(
      "KoH is disabled".to_owned(),
    ));
  }
  let env = challenge_bucket
    .env()
    .await?
    .ok_or(ResponseError::PreconditionFailed(
      "KoH shared hill requires an online environment".to_owned(),
    ))?;
  if env.images.is_empty() || env.images.iter().all(|image| image.port.is_none()) {
    return Err(ResponseError::PreconditionFailed(
      "KoH shared hill requires at least one service port".to_owned(),
    ));
  }
  let snapshot = koh_util::ensure_hill_env(
    &state.cluster,
    &state.config,
    &config,
    &game,
    &challenge,
    env,
  )
  .await?;
  Ok(Json(snapshot.map(|snapshot| snapshot.pod)))
}

pub(super) async fn stop_koh_hill(
  State(state): State<GlobalState>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    koh_util::stop_hill_env(&state.cluster, &challenge)
      .await?
      .into_iter()
      .map(|snapshot| snapshot.pod)
      .collect::<Vec<_>>(),
  ))
}

pub(super) async fn check_koh_once(
  State(state): State<GlobalState>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  worker::koh::check_once(&state, &game, &challenge, true).await?;
  Ok(())
}

pub(super) async fn get_koh_events(
  State(ref db): State<Database>, Extension(challenge): Extension<challenge::Model>,
  Query(query): Query<EventQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    koh_event::get_list(
      &db.conn,
      challenge.id,
      query.limit.unwrap_or(50).clamp(1, 200),
    )
    .await?,
  ))
}

pub(super) async fn get_koh_scoreboard(
  State(ref db): State<Database>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let awards = koh_award::get_list_ex(&db.conn, challenge.id).await?;
  let mut scores = HashMap::<i64, KohScore>::new();
  for award in awards {
    let entry = scores.entry(award.team_id).or_insert(KohScore {
      team_id: award.team_id,
      team_name: award.team_name.clone(),
      score: 0,
      last_awarded_at: None,
    });
    entry.score += award.score;
    if entry
      .last_awarded_at
      .is_none_or(|last| last < award.created_at)
    {
      entry.last_awarded_at = Some(award.created_at);
    }
  }
  let mut scores = scores.into_values().collect::<Vec<_>>();
  scores.sort_by(|a, b| {
    b.score
      .cmp(&a.score)
      .then_with(|| a.last_awarded_at.cmp(&b.last_awarded_at))
  });
  Ok(Json(scores))
}
