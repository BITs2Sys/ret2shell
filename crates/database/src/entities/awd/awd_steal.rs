//! `SeaORM` Entity: an AWD attack (attacker submitted victim's round flag).

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QueryOrder, QuerySelect,
  entity::prelude::*,
};
use serde::{Deserialize, Serialize};

use super::super::team;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awd_steal")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub round: i64,
  pub attacker_team_id: i64,
  pub victim_team_id: i64,
  pub score: i32,
  pub extra_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct ExModel {
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub round: i64,
  pub attacker_team_id: i64,
  pub attacker_name: Option<String>,
  pub victim_team_id: i64,
  pub score: i32,
  pub extra_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::super::team::Entity",
    from = "Column::AttackerTeamId",
    to = "super::super::team::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Attacker,
}

impl Related<super::super::team::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Attacker.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// Whether this attacker already stole this victim's flag this round (idempotency).
pub async fn exists<C>(
  db: &C, challenge_id: i64, round: i64, attacker: i64, victim: i64,
) -> Result<bool, DbErr>
where
  C: ConnectionTrait, {
  Ok(
    Entity::find()
      .filter(Column::ChallengeId.eq(challenge_id))
      .filter(Column::Round.eq(round))
      .filter(Column::AttackerTeamId.eq(attacker))
      .filter(Column::VictimTeamId.eq(victim))
      .one(db)
      .await?
      .is_some(),
  )
}

/// Distinct number of teams that have ever been attacked's-flag stolen for this
/// challenge — i.e. how many teams got exploited, drives the decay.
pub async fn distinct_victim_count<C>(db: &C, challenge_id: i64) -> Result<u64, DbErr>
where
  C: ConnectionTrait, {
  let rows = Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .select_only()
    .column(Column::VictimTeamId)
    .distinct()
    .into_tuple::<i64>()
    .all(db)
    .await?;
  Ok(rows.len() as u64)
}

/// Whether a team has ever had its flag stolen for this challenge — i.e. whether it is
/// already counted in the distinct-victim decay set.
pub async fn victim_exploited<C>(db: &C, challenge_id: i64, victim: i64) -> Result<bool, DbErr>
where
  C: ConnectionTrait, {
  Ok(
    Entity::find()
      .filter(Column::ChallengeId.eq(challenge_id))
      .filter(Column::VictimTeamId.eq(victim))
      .one(db)
      .await?
      .is_some(),
  )
}

pub async fn get_list_ex<C>(db: &C, challenge_id: i64) -> Result<Vec<ExModel>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .join(JoinType::LeftJoin, Relation::Attacker.def())
    .filter(Column::ChallengeId.eq(challenge_id))
    .column_as(team::Column::Name, "attacker_name")
    .order_by_desc(Column::CreatedAt)
    .into_model()
    .all(db)
    .await
}

pub async fn create<C>(db: &C, steal: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..steal.into_active_model().reset_all()
  }
  .insert(db)
  .await
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
