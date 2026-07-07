//! `SeaORM` Entity: records the round a team first solved/fixed an AWDP challenge.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{ActiveValue, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awdp_solve")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  /// the round in which this team first solved (bonus accrues from here on).
  pub solved_round: i64,
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
    .all(db)
    .await
}

/// Record a first solve; a no-op if the team already has one.
pub async fn record<C>(
  db: &C, challenge_id: i64, team_id: i64, solved_round: i64,
) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  if let Some(existing) = get_for_team(db, challenge_id, team_id).await? {
    return Ok(existing);
  }
  let model = Model {
    id: 0,
    created_at: Utc::now(),
    challenge_id,
    team_id,
    solved_round,
  };
  ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..model.into_active_model().reset_all()
  }
  .insert(db)
  .await
}
