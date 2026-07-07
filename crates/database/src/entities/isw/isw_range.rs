//! `SeaORM` Entity for a concrete ISW range instance placed on a host.

use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_range")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub template_id: i64,
  pub host_id: i64,
  /// which team-group this range serves (0-based), e.g. 5 teams per group.
  pub group_index: i32,
  pub name: String,
  /// "pending" | "provisioning" | "armed" | "error" | "down".
  pub status: String,
  #[serde(with = "ts_seconds_option")]
  pub armed_at: Option<DateTime<Utc>>,
  /// clean baseline snapshot name, e.g. "clean-armed".
  pub snapshot_name: Option<String>,
  pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::isw_range_template::Entity",
    from = "Column::TemplateId",
    to = "super::isw_range_template::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Template,
  #[sea_orm(
    belongs_to = "super::isw_host::Entity",
    from = "Column::HostId",
    to = "super::isw_host::Column::Id",
    on_update = "Cascade",
    on_delete = "Restrict"
  )]
  Host,
}

impl Related<super::isw_range_template::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Template.def()
  }
}

impl Related<super::isw_host::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Host.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn list_by_template<C>(db: &C, template_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::TemplateId.eq(template_id))
    .order_by_asc(Column::GroupIndex)
    .all(db)
    .await
}

pub async fn list_by_host<C>(db: &C, host_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::HostId.eq(host_id))
    .order_by_asc(Column::Id)
    .all(db)
    .await
}

pub async fn create<C>(db: &C, range: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let range = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..range.into_active_model().reset_all()
  };
  range.insert(db).await
}

/// Update mutable runtime state (status/armed_at/snapshot/last_error) only.
pub async fn update_state<C>(db: &C, range: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let range = ActiveModel {
    id: ActiveValue::Unchanged(range.id),
    status: ActiveValue::Set(range.status),
    armed_at: ActiveValue::Set(range.armed_at),
    snapshot_name: ActiveValue::Set(range.snapshot_name),
    last_error: ActiveValue::Set(range.last_error),
    ..Default::default()
  };
  range.update(db).await
}

pub async fn delete<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
