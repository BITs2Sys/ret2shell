use axum::{
  extract::{Query, State},
  middleware,
  response::IntoResponse,
  routing::{get, patch, post},
  Extension, Json, Router,
};
use chrono::{serde::ts_seconds, DateTime, Utc};
use nanoid::nanoid;
use r2s_bucket::Bucket;
use r2s_cache::Cache;
use r2s_cluster::{Cluster, Pod, CHALLENGE_NS};
use r2s_database::{
  article, game, submission, team as team_db,
  user::{self, Permission},
};
use r2s_event::{
  events::{EventContainer, GameEvent, GameEventType},
  Event, EventManager,
};
use r2s_migrator::Database;
use r2s_queue::Queue;
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
  middleware::{
    auth::{self, is_game_admin, Token},
    data::{self, extract_team},
  },
  traits::{GlobalState, ResponseError},
};

mod challenge;
mod chat;
mod notification;
mod team;
mod worker;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  tokio::spawn(worker::spawn_game_workers(state.clone()));
  Router::new()
    .route("/", post(create_game))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Host
    )))
    .nest(
      "/:game",
      Router::new()
        .route(
          "/administrator",
          get(get_game_administrator).patch(update_game_administrator),
        )
        .route("/device", get(get_connected_devices))
        .route("/introduction", patch(update_game_intro))
        .route("/", patch(update_game).delete(delete_game))
        .route_layer(middleware::from_fn(auth::game_admin_required))
        .route("/solve", get(get_self_solves))
        .route(
          "/env",
          get(get_self_envs)
            .patch(delay_self_env)
            .delete(stop_self_env),
        )
        .nest("/challenge", challenge::router(state))
        .nest("/team", team::router(state))
        .nest("/notification", notification::router(state))
        .nest("/chat", chat::router(state))
        .route("/introduction", get(get_game_intro))
        .route("/", get(get_game))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_team_info,
        ))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_data!(game, true),
        )),
    )
    .route("/", get(get_game_list))
}

macro_rules! get_game_bucket_mut {
  ($bucket:expr, $game: expr) => {{
    $bucket
      .at_mut(
        &$game
          .bucket
          .clone()
          .ok_or(ResponseError::InternalServerError(
            "bucket is not exist for this game".into(),
            format!(
              "game {}:'{}' does not have a valid bucket",
              $game.id, $game.name
            ),
          ))?,
      )
      .await?
  }};
}

#[derive(Deserialize)]
struct GameListQuery {
  page: Option<u64>,
  page_size: Option<u64>,
  host_type: Option<game::HostType>,
  weight: Option<i32>,
}

async fn get_game_list(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Query(query): Query<GameListQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let results = game::get_page(
    &db.conn,
    query.page.unwrap_or(1),
    query.page_size.unwrap_or(15),
    query.host_type,
    query.weight,
    token.permissions.0.contains(&Permission::Host)
      || token.permissions.0.contains(&Permission::Game),
  )
  .await?;
  Ok(Json((
    results
      .0
      .iter()
      .filter(|g| !g.hidden || g.admins.0.contains(&token.id))
      .cloned()
      .collect::<Vec<_>>(),
    results.1,
  )))
}

async fn get_game(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if game.hidden && !is_game_admin!(token, game) {
    warn!(
      "unauthorized user {}:'{}' ({}) trying to get a hidden game {}:'{}'",
      token.id, token.account, token.nickname, game.id, game.name
    );
    return Err(ResponseError::NotFound("game not found".to_owned()));
  }
  if is_game_admin!(token, game) {
    Ok(Json(game))
  } else {
    Ok(Json(game.desensitize()))
  }
}

async fn create_game(
  State(ref db): State<Database>, State(ref bucket): State<Bucket>,
  Extension(token): Extension<Token>, Json(mut model): Json<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let game_bucket = bucket.create(serde_json::to_value(&model)?).await?;
  model.bucket = Some(game_bucket.name.clone());
  let model = game::create(
    &txn,
    game::Model {
      admins: game::Admins(vec![token.id]),
      introduction_id: None,
      token: Some(nanoid!()),
      ..model
    },
  )
  .await;

  match model {
    Ok(model) => {
      txn.commit().await?;
      Ok(Json(model))
    }
    Err(e) => {
      bucket.delete(&game_bucket.name).await.ok();
      Err(e)?
    }
  }
}

async fn update_game(
  State(ref db): State<Database>, State(ref cache): State<Cache>, State(ref queue): State<Queue>,
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(token): Extension<Token>, Json(model): Json<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let model = game::update(
    &txn,
    game::Model {
      id: game.id,
      bucket: game.bucket.clone(),
      introduction_id: game.introduction_id,
      ..model
    },
  )
  .await?;
  cache.at("game").del(game.id).await?;
  let game_bucket = get_game_bucket_mut!(bucket, model);
  game_bucket
    .set_config(serde_json::to_value(&model)?)
    .await?;
  game_bucket
    .commit(
      ":construction: update game config",
      "platform",
      "platform@private.ret.sh.cn",
    )
    .await?;
  txn.commit().await?;
  if game.frozen != model.frozen {
    let payload = EventContainer {
      game_id: game.id,
      event: Event::Game(GameEvent {
        event_type: if model.frozen {
          GameEventType::Freeze
        } else {
          GameEventType::Unfreeze
        },
        operator: user::Model {
          id: token.id,
          account: token.account.clone(),
          nickname: token.nickname.clone(),
          ..Default::default()
        },
        message: format!(
          "{} the game",
          if model.frozen { "Freeze" } else { "Unfreeze" }
        ),
      }),
    };
    queue.publish("event", payload).await.ok();
  }
  Ok(Json(model))
}

#[derive(Deserialize)]
pub struct DeleteGameQuery {
  force: Option<bool>,
}

async fn delete_game(
  State(ref db): State<Database>, State(ref cache): State<Cache>, State(ref bucket): State<Bucket>,
  Extension(game): Extension<game::Model>, Query(query): Query<DeleteGameQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let delete_result = bucket.delete(&game.bucket.clone().unwrap()).await;
  if !query.force.unwrap_or(false) {
    delete_result?;
  }
  cache.at("game").del(game.id).await?;
  game::delete(&db.conn, game.id).await?;
  Ok(())
}

async fn get_game_intro(
  State(ref db): State<Database>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if let Some(intro_id) = game.introduction_id {
    let intro = article::get_ex(&db.conn, intro_id).await?;
    Ok(Json(intro))
  } else {
    Err(ResponseError::NotFound("introduction not found".to_owned()))
  }
}

async fn update_game_intro(
  State(ref db): State<Database>, State(ref cache): State<Cache>, State(ref bucket): State<Bucket>,
  Extension(game): Extension<game::Model>, Extension(token): Extension<Token>,
  Json(model): Json<article::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let result = if let Some(intro_id) = game.introduction_id {
    article::update(
      &txn,
      intro_id,
      article::Model {
        id: intro_id,
        publisher_id: token.id,
        ..model
      },
    )
    .await?
  } else {
    let model = article::create(
      &txn,
      article::Model {
        id: 0,
        publisher_id: token.id,
        ..model
      },
    )
    .await?;
    game::update(
      &txn,
      game::Model {
        id: game.id,
        introduction_id: Some(model.id),
        ..game.clone()
      },
    )
    .await?;
    cache.at("game").del(game.id).await?;
    model
  };

  let game_bucket = get_game_bucket_mut!(bucket, game);
  game_bucket
    .set_introduction(&result.clone().content.unwrap_or("NO CONTENT".into()))
    .await?;
  game_bucket
    .commit(
      ":memo: update README.md",
      "platform",
      "platform@private.ret.sh.cn",
    )
    .await?;
  txn.commit().await?;

  Ok(Json(result))
}

#[derive(Serialize, Clone, Deserialize)]
struct Instance {
  pub state: String,
  pub name: String,
  pub wsrx: String,
  pub ports: Vec<u16>,
  pub renew_count: i32,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub user_id: i64,
  pub user_name: String,
  pub team_id: i64,
  pub team_name: String,
  pub challenge_id: i64,
  pub challenge_name: String,
  pub game_id: i64,
  pub game_name: String,
}

macro_rules! get_pod_field {
  ($pod:ident, $domain:tt, $field:expr) => {{
    $pod
      .metadata
      .$domain
      .clone()
      .ok_or(ResponseError::Gone(format!(
        "pod field not found: {}",
        $field
      )))?
      .get($field)
      .map(|s| s.clone())
      .ok_or(ResponseError::Gone(format!(
        "pod field not found: {}",
        $field
      )))?
  }};
}

impl TryFrom<Pod> for Instance {
  type Error = ResponseError;
  fn try_from(value: Pod) -> Result<Self, Self::Error> {
    Ok(Instance {
      state: value
        .status
        .map(|s| s.phase.unwrap_or("Unknown".to_string()))
        .ok_or(ResponseError::Gone("pod status not found".to_owned()))?
        .clone(),
      name: value.metadata.name.clone().unwrap_or_default(),
      wsrx: get_pod_field!(value, labels, "ret.sh.cn/wsrx"),
      ports: get_pod_field!(value, annotations, "ret.sh.cn/ports")
        .split(',')
        .map(|p| p.parse().unwrap_or(0))
        .collect(),
      renew_count: get_pod_field!(value, annotations, "ret.sh.cn/renew")
        .parse()
        .map_err(|_| ResponseError::Gone("renew count not found".to_owned()))?,
      created_at: value
        .metadata
        .creation_timestamp
        .clone()
        .map(|c| c.0)
        .ok_or(ResponseError::Gone(
          "pod creation time not found".to_owned(),
        ))?,
      user_id: get_pod_field!(value, labels, "ret.sh.cn/user")
        .parse()
        .map_err(|_| ResponseError::Gone("user id not found".to_owned()))?,
      user_name: get_pod_field!(value, annotations, "ret.sh.cn/user").clone(),
      team_id: get_pod_field!(value, labels, "ret.sh.cn/team")
        .parse()
        .unwrap_or(0),
      team_name: get_pod_field!(value, annotations, "ret.sh.cn/team").clone(),
      challenge_id: get_pod_field!(value, labels, "ret.sh.cn/challenge")
        .parse()
        .map_err(|_| ResponseError::Gone("challenge id not found".to_owned()))?,
      challenge_name: get_pod_field!(value, annotations, "ret.sh.cn/challenge").clone(),
      game_id: get_pod_field!(value, labels, "ret.sh.cn/game")
        .parse()
        .map_err(|_| ResponseError::Gone("game id not found".to_owned()))?,
      game_name: get_pod_field!(value, annotations, "ret.sh.cn/game"),
    })
  }
}

async fn get_self_solves(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, team_ext: Option<Extension<team_db::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  if is_game_admin!(token, game) {
    let solves =
      submission::get_list(&db.conn, true, false, None, None, Some(token.id), false).await?;
    return Ok(Json(solves));
  }
  let team = extract_team!(game, team_ext, token);
  let solves = submission::get_list(
    &db.conn,
    true,
    false,
    None,
    team.clone().map(|t| t.id),
    if team.is_none() { Some(token.id) } else { None },
    false,
  )
  .await?;
  Ok(Json(solves))
}

async fn get_self_envs(
  State(cluster): State<Cluster>, Extension(game): Extension<game::Model>,
  Extension(token): Extension<Token>, team_ext: Option<Extension<team_db::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  let self_env = cluster
    .at(CHALLENGE_NS)
    .get_challenge_env_by_user(token.id)
    .await?;
  let team_envs = if let Some(team) = team {
    cluster
      .at(CHALLENGE_NS)
      .get_challenge_env_by_team(team.id)
      .await?
  } else {
    vec![]
  };
  let mut envs: Vec<Instance> = vec![];
  if let Some(self_env) = self_env {
    envs.push(self_env.try_into()?);
  }
  envs.extend(
    team_envs
      .into_iter()
      .map(|env| env.try_into())
      .collect::<Result<Vec<_>, _>>()?,
  );

  Ok(Json(envs))
}

async fn delay_self_env(
  State(cluster): State<Cluster>, Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, ResponseError> {
  cluster
    .at(CHALLENGE_NS)
    .delay_challenge_env(token.id)
    .await?;
  Ok(())
}

async fn stop_self_env(
  State(cluster): State<Cluster>, Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, ResponseError> {
  cluster
    .at(CHALLENGE_NS)
    .stop_challenge_env(token.id)
    .await?;
  Ok(())
}

#[derive(Serialize)]
struct ConnectedDevice {
  client: String,
  address: String,
  #[serde(with = "ts_seconds")]
  connected_at: DateTime<Utc>,
}

async fn get_connected_devices(
  State(event): State<EventManager>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let clients = event.clients.read().await;
  let clients = clients
    .iter()
    .filter(|(id, _, _, _)| game.id == *id)
    .map(|(_, c, a, d)| ConnectedDevice {
      client: c.clone(),
      address: a.to_string(),
      connected_at: *d,
    })
    .collect::<Vec<_>>();
  Ok(Json(clients))
}

async fn get_game_administrator(
  State(ref db): State<Database>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let admins = user::get_multiple(&db.conn, &game.admins.0).await?;
  Ok(Json(admins))
}

async fn update_game_administrator(
  State(ref db): State<Database>, State(ref cache): State<Cache>,
  Extension(game): Extension<game::Model>, Json(admins): Json<Vec<i64>>,
) -> Result<impl IntoResponse, ResponseError> {
  let model = game::update(
    &db.conn,
    game::Model {
      id: game.id,
      admins: game::Admins(admins),
      ..game.clone()
    },
  )
  .await?;
  cache.at("game").del(game.id).await?;
  Ok(Json(model))
}
