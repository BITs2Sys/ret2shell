use chrono::Utc;
use r2s_cache::Cache;
use r2s_cluster::{
  ChallengeEnvSnapshot, Cluster, Pod,
  lifecycle::{
    LifecycleChallengeInfo, LifecycleEvent, LifecycleExecutionStatus, LifecycleStopReason,
    LifecycleTeamInfo, LifecycleUserInfo,
  },
};
use r2s_database::{challenge, config, game, team, user};
use r2s_engine::Engine;
use tracing::{error, info, warn};

use crate::{middleware::auth::Token, traits::GlobalState};

#[derive(Clone, Copy, Debug)]
enum ScriptScope {
  Global,
  Game,
}

impl ScriptScope {
  const fn as_str(self) -> &'static str {
    match self {
      Self::Global => "global",
      Self::Game => "game",
    }
  }
}

#[derive(Clone, Debug)]
struct ResolvedLifecycleScript {
  key: String,
  script: String,
  scope: ScriptScope,
}

pub fn user_info_from_token(token: &Token) -> LifecycleUserInfo {
  LifecycleUserInfo {
    id: token.id,
    account: token.account.clone(),
    nickname: token.nickname.clone(),
    institute_id: None,
  }
}

pub fn user_info_from_model(user: &user::Model) -> LifecycleUserInfo {
  LifecycleUserInfo {
    id: user.id,
    account: user.account.clone(),
    nickname: user.nickname.clone(),
    institute_id: user.institute_id,
  }
}

pub fn team_info_from_model(team: &team::Model) -> LifecycleTeamInfo {
  LifecycleTeamInfo {
    id: Some(team.id),
    name: Some(team.name.clone()),
    institute_id: team.institute_id,
    token: team.token.clone(),
  }
}

pub fn challenge_info_from_model(challenge: &challenge::Model) -> LifecycleChallengeInfo {
  LifecycleChallengeInfo {
    id: challenge.id,
    name: challenge.name.clone(),
    game_id: challenge.game_id,
  }
}

fn pod_label(pod: &Pod, key: &str) -> Option<String> {
  pod.metadata.labels.as_ref()?.get(key).cloned()
}

fn pod_annotation(pod: &Pod, key: &str) -> Option<String> {
  pod.metadata.annotations.as_ref()?.get(key).cloned()
}

fn fallback_user_info(snapshot: &ChallengeEnvSnapshot) -> LifecycleUserInfo {
  LifecycleUserInfo {
    id: pod_label(&snapshot.pod, "ret.sh.cn/user")
      .and_then(|value| value.parse::<i64>().ok())
      .unwrap_or_default(),
    account: pod_annotation(&snapshot.pod, "ret.sh.cn/user").unwrap_or_default(),
    nickname: pod_annotation(&snapshot.pod, "ret.sh.cn/user-nickname").unwrap_or_default(),
    institute_id: None,
  }
}

fn fallback_team_info(snapshot: &ChallengeEnvSnapshot) -> Option<LifecycleTeamInfo> {
  let id = pod_label(&snapshot.pod, "ret.sh.cn/team")
    .and_then(|value| value.parse::<i64>().ok())
    .filter(|value| *value > 0);
  if id.is_none() {
    return None;
  }
  Some(LifecycleTeamInfo {
    id,
    name: pod_annotation(&snapshot.pod, "ret.sh.cn/team"),
    institute_id: None,
    token: None,
  })
}

fn fallback_challenge_info(
  snapshot: &ChallengeEnvSnapshot, game_id: i64,
) -> LifecycleChallengeInfo {
  LifecycleChallengeInfo {
    id: pod_label(&snapshot.pod, "ret.sh.cn/challenge")
      .and_then(|value| value.parse::<i64>().ok())
      .unwrap_or_default(),
    name: pod_annotation(&snapshot.pod, "ret.sh.cn/challenge").unwrap_or_default(),
    game_id,
  }
}

async fn load_platform_config(state: &GlobalState) -> Option<config::Model> {
  if let Ok(Some(config)) = state.cache.at("platform").get("config").await {
    return Some(config);
  }
  match config::get(&state.db.conn).await {
    Ok(dynamic_config) => {
      let config = dynamic_config
        .unwrap_or_default()
        .merge(state.config.clone());
      state.cache.at("platform").set("config", &config).await.ok();
      Some(config)
    }
    Err(err) => {
      error!(error=?err, "failed to load platform config for lifecycle hooks");
      None
    }
  }
}

fn resolve_lifecycle_script(
  config: &config::Model, game: &game::Model,
) -> Result<Option<ResolvedLifecycleScript>, String> {
  let Some(cluster_config) = &config.cluster else {
    return Ok(None);
  };
  if game.archive_at > Utc::now() {
    if let Some(lifecycle) = game.lifecycle.clone() {
      let key = game.bucket.clone().ok_or_else(|| {
        format!(
          "game {}:{} missing bucket for lifecycle",
          game.id, game.name
        )
      })?;
      return Ok(Some(ResolvedLifecycleScript {
        key,
        script: lifecycle,
        scope: ScriptScope::Game,
      }));
    }
  }
  Ok(Some(ResolvedLifecycleScript {
    key: "default".to_owned(),
    script: cluster_config.lifecycle.clone().unwrap_or_default(),
    scope: ScriptScope::Global,
  }))
}

async fn cleanup_traffic_cache(cache: Cache, snapshots: &[ChallengeEnvSnapshot]) {
  for snapshot in snapshots {
    if let Some(traffic) = pod_label(&snapshot.pod, "ret.sh.cn/traffic") {
      cache.at("traffic").del(traffic).await.ok();
    }
  }
}

async fn trigger_lifecycle_for_snapshots(
  cluster: Cluster, engine: Engine, config: config::Model, game: game::Model,
  challenge: LifecycleChallengeInfo, user: LifecycleUserInfo, team: Option<LifecycleTeamInfo>,
  snapshots: Vec<ChallengeEnvSnapshot>, event: LifecycleEvent, trace_id: Option<String>,
) {
  let Some(mapper) = cluster.lifecycle.clone() else {
    error!(event=%event.name(), "lifecycle mapper is not initialized");
    return;
  };
  let resolved = match resolve_lifecycle_script(&config, &game) {
    Ok(resolved) => resolved,
    Err(err) => {
      match trace_id.as_deref() {
        Some(trace_id) => {
          error!(trace_id=%trace_id, event=%event.name(), game_id=%game.id, challenge_id=%challenge.id, error=%err, "failed to resolve lifecycle script");
        }
        None => {
          error!(event=%event.name(), game_id=%game.id, challenge_id=%challenge.id, error=%err, "failed to resolve lifecycle script");
        }
      }
      return;
    }
  };
  let Some(resolved) = resolved else {
    for snapshot in snapshots {
      let pod_name = snapshot.pod.metadata.name.clone().unwrap_or_default();
      match trace_id.as_deref() {
        Some(trace_id) => {
          info!(trace_id=%trace_id, event=%event.name(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, outcome="skipped", reason="cluster config missing", "lifecycle hook skipped");
        }
        None => {
          info!(event=%event.name(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, outcome="skipped", reason="cluster config missing", "lifecycle hook skipped");
        }
      }
    }
    return;
  };
  if resolved.script.trim().is_empty() {
    for snapshot in snapshots {
      let pod_name = snapshot.pod.metadata.name.clone().unwrap_or_default();
      match trace_id.as_deref() {
        Some(trace_id) => {
          info!(trace_id=%trace_id, event=%event.name(), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, outcome="skipped", reason="script empty", "lifecycle hook skipped");
        }
        None => {
          info!(event=%event.name(), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, outcome="skipped", reason="script empty", "lifecycle hook skipped");
        }
      }
    }
    return;
  }
  if let Err(err) = mapper
    .preload(&engine, &resolved.key, &resolved.script)
    .await
  {
    match trace_id.as_deref() {
      Some(trace_id) => {
        error!(trace_id=%trace_id, event=%event.name(), scope=%resolved.scope.as_str(), game_id=%game.id, challenge_id=%challenge.id, error=?err, "failed to preload lifecycle script");
      }
      None => {
        error!(event=%event.name(), scope=%resolved.scope.as_str(), game_id=%game.id, challenge_id=%challenge.id, error=?err, "failed to preload lifecycle script");
      }
    }
    return;
  }
  for snapshot in snapshots {
    let pod_name = snapshot.pod.metadata.name.clone().unwrap_or_default();
    let team_id = team.as_ref().and_then(|team| team.id).unwrap_or_default();
    let team_name = team
      .as_ref()
      .and_then(|team| team.name.clone())
      .unwrap_or_default();
    match mapper
      .execute(
        &engine,
        &resolved.key,
        event,
        &snapshot,
        user.clone(),
        team.clone(),
        challenge.clone(),
      )
      .await
    {
      Ok(LifecycleExecutionStatus::Executed) => match trace_id.as_deref() {
        Some(trace_id) => {
          info!(trace_id=%trace_id, event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, outcome="executed", "lifecycle hook executed");
        }
        None => {
          info!(event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, outcome="executed", "lifecycle hook executed");
        }
      },
      Ok(LifecycleExecutionStatus::Skipped) => match trace_id.as_deref() {
        Some(trace_id) => {
          info!(trace_id=%trace_id, event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, outcome="skipped", reason_detail="function missing", "lifecycle hook skipped");
        }
        None => {
          info!(event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, outcome="skipped", reason_detail="function missing", "lifecycle hook skipped");
        }
      },
      Err(err) => match trace_id.as_deref() {
        Some(trace_id) => {
          error!(trace_id=%trace_id, event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, error=?err, outcome="failed", "lifecycle hook failed");
        }
        None => {
          error!(event=%event.name(), reason=%event.reason().map(|reason| reason.as_str()).unwrap_or(""), scope=%resolved.scope.as_str(), pod=%pod_name, game_id=%game.id, challenge_id=%challenge.id, user_id=%user.id, team_id, team_name=%team_name, error=?err, outcome="failed", "lifecycle hook failed");
        }
      },
    }
  }
}

pub fn spawn_request_hooks(
  cache: Option<Cache>, cluster: Cluster, engine: Engine, config: config::Model, game: game::Model,
  challenge: challenge::Model, token: Token, team: Option<team::Model>,
  snapshots: Vec<ChallengeEnvSnapshot>, event: LifecycleEvent, trace_id: String,
) {
  if snapshots.is_empty() {
    return;
  }
  let user = user_info_from_token(&token);
  let challenge = challenge_info_from_model(&challenge);
  let team = team.as_ref().map(team_info_from_model);
  tokio::spawn(async move {
    if let Some(cache) = cache {
      cleanup_traffic_cache(cache, &snapshots).await;
    }
    trigger_lifecycle_for_snapshots(
      cluster,
      engine,
      config,
      game,
      challenge,
      user,
      team,
      snapshots,
      event,
      Some(trace_id),
    )
    .await;
  });
}

pub fn spawn_timeout_stop_hooks(state: GlobalState, snapshots: Vec<ChallengeEnvSnapshot>) {
  if snapshots.is_empty() {
    return;
  }
  tokio::spawn(async move {
    cleanup_traffic_cache(state.cache.clone(), &snapshots).await;
    let Some(config) = load_platform_config(&state).await else {
      return;
    };
    for snapshot in snapshots {
      let Some(game_id) =
        pod_label(&snapshot.pod, "ret.sh.cn/game").and_then(|value| value.parse::<i64>().ok())
      else {
        warn!(pod=?snapshot.pod.metadata.name, "missing game id for timeout lifecycle hook");
        continue;
      };
      let game = match game::get(&state.db.conn, game_id).await {
        Ok(Some(game)) => game,
        Ok(None) => {
          warn!(game_id, pod=?snapshot.pod.metadata.name, "game missing for timeout lifecycle hook");
          continue;
        }
        Err(err) => {
          error!(game_id, pod=?snapshot.pod.metadata.name, error=?err, "failed to load game for timeout lifecycle hook");
          continue;
        }
      };
      let challenge = match pod_label(&snapshot.pod, "ret.sh.cn/challenge")
        .and_then(|value| value.parse::<i64>().ok())
      {
        Some(challenge_id) => match challenge::get(&state.db.conn, challenge_id).await {
          Ok(Some(challenge)) => challenge_info_from_model(&challenge),
          Ok(None) => fallback_challenge_info(&snapshot, game.id),
          Err(err) => {
            error!(challenge_id, game_id=%game.id, pod=?snapshot.pod.metadata.name, error=?err, "failed to load challenge for timeout lifecycle hook");
            fallback_challenge_info(&snapshot, game.id)
          }
        },
        None => fallback_challenge_info(&snapshot, game.id),
      };
      let user = match pod_label(&snapshot.pod, "ret.sh.cn/user")
        .and_then(|value| value.parse::<i64>().ok())
      {
        Some(user_id) => match user::get(&state.db.conn, user_id).await {
          Ok(Some(user)) => user_info_from_model(&user),
          Ok(None) => fallback_user_info(&snapshot),
          Err(err) => {
            error!(user_id, game_id=%game.id, pod=?snapshot.pod.metadata.name, error=?err, "failed to load user for timeout lifecycle hook");
            fallback_user_info(&snapshot)
          }
        },
        None => fallback_user_info(&snapshot),
      };
      let team = match pod_label(&snapshot.pod, "ret.sh.cn/team")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
      {
        Some(team_id) => match team::get(&state.db.conn, team_id).await {
          Ok(Some(team)) => Some(team_info_from_model(&team)),
          Ok(None) => fallback_team_info(&snapshot),
          Err(err) => {
            error!(team_id, game_id=%game.id, pod=?snapshot.pod.metadata.name, error=?err, "failed to load team for timeout lifecycle hook");
            fallback_team_info(&snapshot)
          }
        },
        None => None,
      };
      trigger_lifecycle_for_snapshots(
        state.cluster.clone(),
        state.engine.clone(),
        config.clone(),
        game,
        challenge,
        user,
        team,
        vec![snapshot],
        LifecycleEvent::Stop(LifecycleStopReason::Timeout),
        None,
      )
      .await;
    }
  });
}
