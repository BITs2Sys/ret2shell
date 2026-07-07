//! `SeaORM` Entity for a minted + injected ISW flag, bound to a challenge.
//!
//! Only the sha256 of the flag value is stored; the plaintext lives nowhere in the
//! database (it is injected into the guest and verified by read-back hash).

use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_flag")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub range_id: i64,
  pub challenge_id: i64,
  /// which VM the flag was injected into (nullable if the VM row is gone).
  pub vm_id: Option<i64>,
  pub guest_path: String,
  /// sha256 hex of the injected flag value.
  pub value_hash: String,
  /// rotation round; 0 for the static/base flag.
  pub round: i32,
  #[serde(with = "ts_seconds_option")]
  pub injected_at: Option<DateTime<Utc>>,
  pub verified: bool,
  pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::isw_range::Entity",
    from = "Column::RangeId",
    to = "super::isw_range::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Range,
  #[sea_orm(
    belongs_to = "super::super::challenge::Entity",
    from = "Column::ChallengeId",
    to = "super::super::challenge::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Challenge,
}

impl Related<super::isw_range::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Range.def()
  }
}

impl Related<super::super::challenge::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Challenge.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn list_by_range<C>(db: &C, range_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .order_by_asc(Column::ChallengeId)
    .all(db)
    .await
}

/// Delete all flags for a range (used before a fresh (re-)arm).
pub async fn clear_range<C>(db: &C, range_id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_many()
    .filter(Column::RangeId.eq(range_id))
    .exec(db)
    .await
    .map(|_| ())
}

pub async fn get_current<C>(
  db: &C, range_id: i64, challenge_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .filter(Column::ChallengeId.eq(challenge_id))
    .order_by_desc(Column::Round)
    .one(db)
    .await
}

/// Insert a freshly minted (not yet verified) flag record.
pub async fn create<C>(db: &C, flag: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let flag = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..flag.into_active_model().reset_all()
  };
  flag.insert(db).await
}

/// Mark a flag as successfully injected + verified.
pub async fn mark_verified<C>(db: &C, id: i64, verified: bool) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let flag = ActiveModel {
    id: ActiveValue::Unchanged(id),
    verified: ActiveValue::Set(verified),
    injected_at: ActiveValue::Set(Some(Utc::now())),
    last_error: ActiveValue::Set(None),
    ..Default::default()
  };
  flag.update(db).await.map(|_| ())
}

pub async fn mark_error<C>(db: &C, id: i64, error: &str) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let flag = ActiveModel {
    id: ActiveValue::Unchanged(id),
    verified: ActiveValue::Set(false),
    last_error: ActiveValue::Set(Some(error.to_owned())),
    ..Default::default()
  };
  flag.update(db).await.map(|_| ())
}
