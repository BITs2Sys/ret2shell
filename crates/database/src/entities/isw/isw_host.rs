//! `SeaORM` Entity for an ISW VMware host running the `r2s-isw-agent`.

use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_host")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  /// human-readable label, e.g. "host-a".
  pub name: String,
  /// reachable address of the host-agent (ip or dns).
  pub address: String,
  /// host-agent https port.
  pub api_port: i32,
  /// guest platform family: "windows" | "linux" (informational; a host may run
  /// mixed guests).
  pub os: String,
  /// pinned sha256 fingerprint of the agent's mTLS server certificate.
  pub fingerprint: Option<String>,
  pub enabled: bool,
  /// last known agent status: "online" | "offline" | "error".
  pub status: String,
  /// free memory reported by the last heartbeat, in MiB.
  pub free_mem_mb: Option<i64>,
  #[serde(with = "ts_seconds_option")]
  pub last_heartbeat: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn list<C>(db: &C) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find().order_by_asc(Column::Id).all(db).await
}

pub async fn create<C>(db: &C, host: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let host = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..host.into_active_model().reset_all()
  };
  host.insert(db).await
}

pub async fn update<C>(db: &C, host: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let host = ActiveModel {
    id: ActiveValue::Unchanged(host.id),
    created_at: ActiveValue::NotSet,
    ..host.into_active_model().reset_all()
  };
  host.update(db).await
}

/// Record a heartbeat: liveness status + free memory + timestamp only.
pub async fn touch_heartbeat<C>(
  db: &C, id: i64, status: &str, free_mem_mb: Option<i64>,
) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let host = ActiveModel {
    id: ActiveValue::Unchanged(id),
    status: ActiveValue::Set(status.to_owned()),
    free_mem_mb: ActiveValue::Set(free_mem_mb),
    last_heartbeat: ActiveValue::Set(Some(Utc::now())),
    ..Default::default()
  };
  host.update(db).await.map(|_| ())
}

pub async fn delete<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
