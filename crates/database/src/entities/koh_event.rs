//! `SeaORM` Entity for King of the Hill check events.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QueryOrder, QuerySelect,
  entity::prelude::*,
};
use serde::{Deserialize, Serialize};

use super::{challenge, team};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "koh_event")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: Option<i64>,
  pub previous_team_id: Option<i64>,
  #[sea_orm(column_type = "Text")]
  pub identifier: Option<String>,
  pub status: String,
  #[sea_orm(column_type = "Text")]
  pub message: Option<String>,
  pub score_delta: i32,
  pub tick: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct ExModel {
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub challenge_name: Option<String>,
  pub team_id: Option<i64>,
  pub team_name: Option<String>,
  pub previous_team_id: Option<i64>,
  pub identifier: Option<String>,
  pub status: String,
  pub message: Option<String>,
  pub score_delta: i32,
  pub tick: Option<i64>,
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
    on_delete = "SetNull"
  )]
  Team,
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

impl ActiveModelBehavior for ActiveModel {}

pub async fn create<C>(db: &C, event: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let event = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..event.into_active_model().reset_all()
  };
  event.insert(db).await
}

pub async fn get_list<C>(db: &C, challenge_id: i64, limit: u64) -> Result<Vec<ExModel>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .join(JoinType::LeftJoin, Relation::Challenge.def())
    .join(JoinType::LeftJoin, Relation::Team.def())
    .filter(Column::ChallengeId.eq(challenge_id))
    .column_as(challenge::Column::Name, "challenge_name")
    .column_as(team::Column::Name, "team_name")
    .order_by_desc(Column::CreatedAt)
    .limit(limit)
    .into_model()
    .all(db)
    .await
}
