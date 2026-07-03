//! `SeaORM` Entity for King of the Hill team identifiers.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromQueryResult, IntoActiveModel, JoinType, QueryOrder, QuerySelect,
  entity::prelude::*,
};
use serde::{Deserialize, Serialize};

use super::{challenge, team};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "koh_identifier")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  pub identifier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromQueryResult)]
pub struct ExModel {
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub challenge_name: Option<String>,
  pub team_id: i64,
  pub team_name: Option<String>,
  pub identifier: String,
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

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn get_by_team<C>(
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

pub async fn get_by_identifier<C>(
  db: &C, challenge_id: i64, identifier: &str,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Identifier.eq(identifier))
    .one(db)
    .await
}

pub async fn get_list_ex<C>(db: &C, challenge_id: i64) -> Result<Vec<ExModel>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .join(JoinType::LeftJoin, Relation::Challenge.def())
    .join(JoinType::LeftJoin, Relation::Team.def())
    .filter(Column::ChallengeId.eq(challenge_id))
    .column_as(challenge::Column::Name, "challenge_name")
    .column_as(team::Column::Name, "team_name")
    .order_by_asc(Column::TeamId)
    .into_model()
    .all(db)
    .await
}

pub async fn create<C>(db: &C, identifier: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let identifier = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..identifier.into_active_model().reset_all()
  };
  identifier.insert(db).await
}
