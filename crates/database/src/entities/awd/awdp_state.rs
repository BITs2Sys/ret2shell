//! `SeaORM` Entity: AWDP round scheduler state (one row per challenge).

use chrono::{DateTime, Utc, serde::ts_seconds_option};
use sea_orm::{ActiveValue, IntoActiveModel, QuerySelect, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awdp_state")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub challenge_id: i64,
  pub last_round: i64,
  #[serde(with = "ts_seconds_option")]
  pub last_checked_at: Option<DateTime<Utc>>,
  #[sea_orm(column_type = "Text")]
  pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub fn empty(challenge_id: i64) -> Model {
  Model {
    challenge_id,
    last_round: 0,
    last_checked_at: None,
    last_error: None,
  }
}

pub async fn get<C>(db: &C, challenge_id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(challenge_id).one(db).await
}

/// Row-locking read used inside the scoring transaction so the worker loop and a
/// forced check serialize per-challenge.
pub async fn get_for_update<C>(db: &C, challenge_id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(challenge_id)
    .lock_exclusive()
    .one(db)
    .await
}

pub async fn put<C>(db: &C, state: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  if get(db, state.challenge_id).await?.is_some() {
    let state = ActiveModel {
      challenge_id: ActiveValue::Unchanged(state.challenge_id),
      ..state.into_active_model().reset_all()
    };
    state.update(db).await
  } else {
    state.into_active_model().reset_all().insert(db).await
  }
}
