//! `SeaORM` Entity: a team's own AWD machine (one pod per team per challenge).

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awd_instance")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  /// k8s pod name backing this team's machine.
  pub pod_name: String,
  /// reachable address other teams attack (host:port), when exposed.
  pub address: Option<String>,
  /// "pending" | "running" | "error".
  pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_for_team<C>(
  db: &C, challenge_id: i64, team_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::TeamId.eq(team_id))
    .one(db)
    .await
}

pub async fn list_by_challenge<C>(db: &C, challenge_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .order_by_asc(Column::TeamId)
    .all(db)
    .await
}

pub async fn create<C>(db: &C, instance: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..instance.into_active_model().reset_all()
  }
  .insert(db)
  .await
}

pub async fn update_state<C>(
  db: &C, id: i64, status: &str, address: Option<String>,
) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::Unchanged(id),
    status: ActiveValue::Set(status.to_owned()),
    address: ActiveValue::Set(address),
    ..Default::default()
  }
  .update(db)
  .await
  .map(|_| ())
}

pub async fn delete_by_challenge<C>(db: &C, challenge_id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_many()
    .filter(Column::ChallengeId.eq(challenge_id))
    .exec(db)
    .await
    .map(|_| ())
}

pub async fn delete_by_id<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
