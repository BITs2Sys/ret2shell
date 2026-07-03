use std::collections::{BTreeMap, HashMap};

use chrono::Utc;
use nanoid::nanoid;
use r2s_cluster::{CHALLENGE_NS, ChallengeEnvSnapshot, Cluster};
use r2s_config::{
  GlobalConfig,
  cluster::{ChallengeEnv, KohConfig, KohMode},
};
use r2s_database::{challenge, game};

use crate::traits::ResponseError;

const LABEL_ALPHABET: [char; 62] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
  'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
  'V', 'W', 'X', 'Y', 'Z',
];

pub fn env_ports(env: &ChallengeEnv) -> Vec<u16> {
  env.images.iter().filter_map(|image| image.port).collect()
}

pub fn infer_agent_port(koh: &r2s_config::cluster::KohConfig, env: &ChallengeEnv) -> Option<u16> {
  koh
    .agent_port
    .or(koh.target_port)
    .or_else(|| env.images.iter().find_map(|image| image.port))
}

pub fn internal_status_url(pod_name: &str, port: u16, path: &str) -> String {
  let path = if path.starts_with('/') {
    path.to_owned()
  } else {
    format!("/{path}")
  };
  format!("http://{pod_name}.{CHALLENGE_NS}.svc:{port}{path}")
}

pub fn node_selector(game: &game::Model, config: &GlobalConfig) -> Option<String> {
  let cluster_config = config.cluster.as_ref()?;
  if game.archive_at > Utc::now() {
    game
      .node_selector
      .clone()
      .or_else(|| cluster_config.node_selector.clone())
  } else {
    cluster_config.node_selector.clone()
  }
  .filter(|node_selector| !node_selector.is_empty())
}

pub fn need_expose(game: &game::Model, config: &GlobalConfig) -> bool {
  let Some(cluster_config) = config.cluster.as_ref() else {
    return false;
  };
  if game.archive_at > Utc::now() {
    game.traffic.is_some() || cluster_config.traffic.is_some()
  } else {
    cluster_config.traffic.is_some()
  }
}

pub fn traffic_script<'a>(
  game: &'a game::Model, config: &'a GlobalConfig,
) -> Result<(String, Option<String>), ResponseError> {
  let cluster_config = config
    .cluster
    .as_ref()
    .ok_or(ResponseError::PreconditionFailed(
      "cluster is disabled".to_owned(),
    ))?;
  if game.archive_at > Utc::now() {
    if let Some(traffic) = game.traffic.clone() {
      Ok((
        game
          .bucket
          .clone()
          .ok_or(ResponseError::PreconditionFailed(
            "game bucket not found".to_owned(),
          ))?,
        Some(traffic),
      ))
    } else {
      Ok(("default".to_owned(), cluster_config.traffic.clone()))
    }
  } else {
    Ok(("default".to_owned(), cluster_config.traffic.clone()))
  }
}

pub async fn ensure_hill_env(
  cluster: &Cluster, config: &GlobalConfig, koh_config: &KohConfig, game: &game::Model,
  challenge: &challenge::Model, env_config: ChallengeEnv,
) -> Result<Option<ChallengeEnvSnapshot>, ResponseError> {
  let cluster = cluster.at(CHALLENGE_NS);
  let existing = cluster.get_koh_hill_env(challenge.id).await?;
  if !existing.is_empty() {
    return Ok(None);
  }

  let ports = env_ports(&env_config)
    .into_iter()
    .map(|port| port.to_string())
    .collect::<Vec<_>>()
    .join(",");
  let traffic = nanoid!(21, &LABEL_ALPHABET);
  let labels = [
    ("ret.sh.cn/koh", "true".to_owned()),
    ("ret.sh.cn/koh-role", "hill".to_owned()),
    ("ret.sh.cn/challenge", challenge.id.to_string()),
    ("ret.sh.cn/game", game.id.to_string()),
    ("ret.sh.cn/team", "0".to_owned()),
    ("ret.sh.cn/user", "0".to_owned()),
    ("ret.sh.cn/traffic", traffic),
    ("ret.sh.cn/internet", env_config.internet.to_string()),
  ]
  .iter()
  .map(|(k, v)| (k.to_string(), v.to_owned()))
  .collect::<BTreeMap<_, _>>();
  let annotations = [
    ("ret.sh.cn/challenge", challenge.name.clone()),
    ("ret.sh.cn/game", game.name.clone()),
    ("ret.sh.cn/team", "KoH".to_owned()),
    ("ret.sh.cn/user", "ret2shell".to_owned()),
    ("ret.sh.cn/user-nickname", "Ret2Shell".to_owned()),
    ("ret.sh.cn/renew", "0".to_owned()),
    ("ret.sh.cn/ports", ports),
  ]
  .iter()
  .map(|(k, v)| (k.to_string(), v.to_owned()))
  .collect::<BTreeMap<_, _>>();
  let koh_mode = match koh_config.mode {
    KohMode::AgentHttp => "agent_http",
    KohMode::RoundRankHttp => "round_rank_http",
    KohMode::GameElo => "game_elo",
  };
  let envs = [
    ("R2S_KOH", "true".to_owned()),
    ("R2S_KOH_CHALLENGE_ID", challenge.id.to_string()),
    ("R2S_KOH_GAME_ID", game.id.to_string()),
    ("R2S_KOH_MODE", koh_mode.to_owned()),
    ("KOH_INTERVAL_SECS", koh_config.interval_secs.to_string()),
    ("KOH_ROUND_SECS", koh_config.round_secs.to_string()),
    ("KOH_TOTAL_ROUNDS", koh_config.total_rounds.to_string()),
    ("KOH_REWARD", koh_config.reward.to_string()),
    ("KOH_RANK_COUNT", koh_config.rank_count.to_string()),
    (
      "KOH_RANK_PERCENTAGES",
      koh_config
        .rank_percentages
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(","),
    ),
  ]
  .iter()
  .map(|(k, v)| (k.to_string(), v.to_owned()))
  .collect::<HashMap<_, _>>();

  Ok(Some(
    cluster
      .create_koh_hill_env(
        challenge.id,
        labels,
        annotations,
        envs,
        env_config,
        node_selector(game, config),
        need_expose(game, config),
      )
      .await?,
  ))
}

pub async fn stop_hill_env(
  cluster: &Cluster, challenge: &challenge::Model,
) -> Result<Vec<ChallengeEnvSnapshot>, ResponseError> {
  Ok(
    cluster
      .at(CHALLENGE_NS)
      .stop_koh_hill_env(challenge.id)
      .await?,
  )
}
