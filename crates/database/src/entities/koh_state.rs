//! `SeaORM` Entity for King of the Hill runtime state.

use chrono::{DateTime, Utc, serde::ts_seconds_option};
use sea_orm::{ActiveValue, IntoActiveModel, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "koh_state")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub challenge_id: i64,
  #[sea_orm(column_type = "Text")]
  pub current_identifier: Option<String>,
  pub current_team_id: Option<i64>,
  #[serde(with = "ts_seconds_option")]
  pub last_checked_at: Option<DateTime<Utc>>,
  #[serde(with = "ts_seconds_option")]
  pub last_awarded_at: Option<DateTime<Utc>>,
  #[sea_orm(column_type = "Text")]
  pub last_error: Option<String>,
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
    from = "Column::CurrentTeamId",
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

pub async fn get<C>(db: &C, challenge_id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(challenge_id).one(db).await
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
    let state = state.into_active_model().reset_all();
    state.insert(db).await
  }
}

pub fn empty(challenge_id: i64) -> Model {
  Model {
    challenge_id,
    current_identifier: None,
    current_team_id: None,
    last_checked_at: None,
    last_awarded_at: None,
    last_error: None,
  }
}
