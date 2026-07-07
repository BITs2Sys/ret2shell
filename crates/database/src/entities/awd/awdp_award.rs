//! `SeaORM` Entity: per-round AWDP bonus award (idempotent per challenge+team+round).

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QueryOrder, QuerySelect,
  entity::prelude::*,
};
use serde::{Deserialize, Serialize};

use super::super::team;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awdp_award")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  pub round: i64,
  pub score: i32,
  pub extra_id: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct ExModel {
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  pub team_name: Option<String>,
  pub round: i64,
  pub score: i32,
  pub extra_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::super::team::Entity",
    from = "Column::TeamId",
    to = "super::super::team::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Team,
}

impl Related<super::super::team::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Team.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_by_round_team<C>(
  db: &C, challenge_id: i64, round: i64, team_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Round.eq(round))
    .filter(Column::TeamId.eq(team_id))
    .one(db)
    .await
}

/// The highest round already awarded to a team for this challenge, if any — the
/// per-team frontier the worker resumes from (so a late-detected solve still gets its
/// earlier rounds back-filled instead of being clamped to a stale global frontier).
pub async fn max_round_for_team<C>(
  db: &C, challenge_id: i64, team_id: i64,
) -> Result<Option<i64>, DbErr>
where
  C: ConnectionTrait, {
  Ok(
    Entity::find()
      .filter(Column::ChallengeId.eq(challenge_id))
      .filter(Column::TeamId.eq(team_id))
      .order_by_desc(Column::Round)
      .one(db)
      .await?
      .map(|award| award.round),
  )
}

pub async fn get_list_ex<C>(db: &C, challenge_id: i64) -> Result<Vec<ExModel>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .join(JoinType::LeftJoin, Relation::Team.def())
    .filter(Column::ChallengeId.eq(challenge_id))
    .column_as(team::Column::Name, "team_name")
    .order_by_desc(Column::CreatedAt)
    .into_model()
    .all(db)
    .await
}

pub async fn create<C>(db: &C, award: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..award.into_active_model().reset_all()
  }
  .insert(db)
  .await
}
