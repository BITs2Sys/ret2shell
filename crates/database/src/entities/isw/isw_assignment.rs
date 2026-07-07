//! `SeaORM` Entity mapping a team (within a group) to its ISW range.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{ActiveValue, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_assignment")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub game_id: i64,
  pub range_id: i64,
  pub team_id: i64,
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
    belongs_to = "super::super::team::Entity",
    from = "Column::TeamId",
    to = "super::super::team::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Team,
}

impl Related<super::isw_range::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Range.def()
  }
}

impl Related<super::super::team::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Team.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

/// The range a team is assigned to for a given game (one per team per game).
pub async fn get_for_team<C>(
  db: &C, game_id: i64, team_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::GameId.eq(game_id))
    .filter(Column::TeamId.eq(team_id))
    .one(db)
    .await
}

pub async fn list_by_range<C>(db: &C, range_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .all(db)
    .await
}

pub async fn list_by_game<C>(db: &C, game_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::GameId.eq(game_id))
    .all(db)
    .await
}

pub async fn create<C>(db: &C, assignment: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let assignment = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..assignment.into_active_model().reset_all()
  };
  assignment.insert(db).await
}

pub async fn delete<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
