//! `SeaORM` Entity for King of the Hill score awards.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QueryOrder, QuerySelect,
  entity::prelude::*,
};
use serde::{Deserialize, Serialize};

use super::team;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "koh_award")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  pub tick: i64,
  pub rank: Option<i32>,
  pub percent: Option<i32>,
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
  pub tick: i64,
  pub rank: Option<i32>,
  pub percent: Option<i32>,
  pub score: i32,
  pub extra_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::challenge::Entity",
    from = "Column::ChallengeId",
    to = "super::challenge::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Challenge,
  #[sea_orm(
    belongs_to = "super::team::Entity",
    from = "Column::TeamId",
    to = "super::team::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Team,
  #[sea_orm(
    belongs_to = "super::extra::Entity",
    from = "Column::ExtraId",
    to = "super::extra::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Extra,
}

impl Related<super::challenge::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Challenge.def()
  }
}

impl Related<super::team::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Team.def()
  }
}

impl Related<super::extra::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Extra.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_by_tick<C>(db: &C, challenge_id: i64, tick: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Tick.eq(tick))
    .one(db)
    .await
}

pub async fn get_by_tick_team<C>(
  db: &C, challenge_id: i64, tick: i64, team_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Tick.eq(tick))
    .filter(Column::TeamId.eq(team_id))
    .one(db)
    .await
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
  let award = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..award.into_active_model().reset_all()
  };
  award.insert(db).await
}
