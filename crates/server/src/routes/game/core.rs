use axum::{
  Extension, Json,
  extract::{Query, State},
  response::IntoResponse,
};
use nanoid::nanoid;
use r2s_bucket::Bucket;
use r2s_cache::Cache;
use r2s_database::{
  article, challenge as challenge_db,
  game::{self, ArchivePolicy},
  user::{self, Permission},
};
use r2s_event::{
  Event,
  events::{EventContainer, GameEvent, GameEventType},
};
use r2s_migrator::Database;
use r2s_queue::Queue;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use tower_http::request_id::RequestId;
use tracing::{info, warn};

use crate::{
  middleware::auth::{Token, is_game_admin},
  traits::ResponseError,
};

#[derive(Deserialize)]
pub(super) struct GameListQuery {
  page: Option<u64>,
  page_size: Option<u64>,
  host_type: Option<game::HostType>,
  weight: Option<i32>,
}

pub(super) async fn get_game_list(
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

pub(super) async fn get_game(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if game.hidden && !is_game_admin!(token, game) {
    warn!("unauthorized user trying to get a hidden game");
    return Err(ResponseError::NotFound("game not found".to_owned()));
  }
  if is_game_admin!(token, game) {
    Ok(Json(game))
  } else {
    Ok(Json(game.desensitize()))
  }
}

pub(super) async fn create_game(
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
      archive_policy: ArchivePolicy::default(),
      ..model
    },
  )
  .await;

  match model {
    Ok(model) => {
      txn.commit().await?;
      info!(game_id=%model.id, game_name=%model.name, "created game");
      Ok(Json(model))
    }
    Err(e) => {
      bucket.delete(&game_bucket.name).await.ok();
      warn!(error=?e, "failed to create game, rolling back bucket creation");
      Err(e)?
    }
  }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn update_game(
  State(ref db): State<Database>, State(ref cache): State<Cache>, State(ref queue): State<Queue>,
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(trace): Extension<RequestId>, Extension(token): Extension<Token>,
  Json(model): Json<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let model = game::update(
    &txn,
    game::Model {
      id: game.id,
      bucket: game.bucket.clone(),
      traffic: game.traffic.clone(),
      node_selector: game.node_selector.clone(),
      introduction_id: game.introduction_id,
      ..model
    },
  )
  .await?;
  cache.at("game").del(game.id).await?;
  let game_bucket = super::get_game_bucket_mut(bucket, &model).await?;
  game_bucket
    .set_config(serde_json::to_value(&model)?)
    .await?;
  game_bucket
    .commit(
      ":construction: update game config",
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  txn.commit().await?;
  if game.frozen != model.frozen {
    info!(
      "user {} the game",
      if model.frozen { "freeze" } else { "unfreeze" },
    );
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
    queue
      .publish(
        "event",
        payload,
        &trace.header_value().to_str().unwrap_or("UNKNOWN"),
      )
      .await
      .ok();
  }
  info!("updated game");
  Ok(Json(model))
}

#[derive(Deserialize)]
pub(super) struct DeleteGameQuery {
  pub force: Option<bool>,
}

pub(super) async fn delete_game(
  State(ref db): State<Database>, State(ref cache): State<Cache>, State(ref bucket): State<Bucket>,
  Extension(game): Extension<game::Model>, Query(query): Query<DeleteGameQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenges = challenge_db::count(&db.conn, Some(game.id), Some(game.host_type), true).await?;
  if challenges > 0 && !query.force.unwrap_or(false) {
    return Err(ResponseError::PreconditionFailed(
      "game has existing challenges, can not be deleted safely".to_owned(),
    ));
  }
  cache.at("game").del(game.id).await?;
  game::delete(&db.conn, game.id).await?;
  let delete_result = bucket.delete(&game.bucket.clone().unwrap()).await;
  if !query.force.unwrap_or(false) {
    delete_result?;
  }
  info!("deleted game");
  Ok(())
}

pub(super) async fn get_game_intro(
  State(ref db): State<Database>, Extension(game): Extension<game::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if let Some(intro_id) = game.introduction_id {
    let intro = article::get_ex(&db.conn, intro_id).await?;
    Ok(Json(intro))
  } else {
    Err(ResponseError::NotFound("introduction not found".to_owned()))
  }
}

pub(super) async fn update_game_intro(
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

  let game_bucket = super::get_game_bucket_mut(bucket, &game).await?;
  game_bucket
    .set_introduction(&result.clone().content.unwrap_or("NO CONTENT".into()))
    .await?;
  game_bucket
    .commit(
      ":memo: update README.md",
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  txn.commit().await?;
  info!("created introduction for game");

  Ok(Json(result))
}
