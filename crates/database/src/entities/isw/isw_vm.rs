//! `SeaORM` Entity for a single guest VM within an ISW range.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_vm")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub range_id: i64,
  pub logical_name: String,
  /// "linux" | "windows".
  pub guest_os: String,
  pub vmx_path: String,
  /// last known guest ip on the range network.
  pub ip: Option<String>,
  /// "on" | "off" | "unknown".
  pub power_state: String,
  /// "running" | "not_running" | "unknown".
  pub tools_state: String,
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
}

impl Related<super::isw_range::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Range.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn list_by_range<C>(db: &C, range_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .order_by_asc(Column::LogicalName)
    .all(db)
    .await
}

pub async fn get_by_logical<C>(
  db: &C, range_id: i64, logical_name: &str,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .filter(Column::LogicalName.eq(logical_name))
    .one(db)
    .await
}

pub async fn create<C>(db: &C, vm: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let vm = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..vm.into_active_model().reset_all()
  };
  vm.insert(db).await
}

/// Update runtime state (ip/power/tools) only.
pub async fn update_state<C>(
  db: &C, id: i64, ip: Option<String>, power_state: &str, tools_state: &str,
) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let vm = ActiveModel {
    id: ActiveValue::Unchanged(id),
    ip: ActiveValue::Set(ip),
    power_state: ActiveValue::Set(power_state.to_owned()),
    tools_state: ActiveValue::Set(tools_state.to_owned()),
    ..Default::default()
  };
  vm.update(db).await.map(|_| ())
}

pub async fn delete_by_range<C>(db: &C, range_id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_many()
    .filter(Column::RangeId.eq(range_id))
    .exec(db)
    .await
    .map(|_| ())
}
