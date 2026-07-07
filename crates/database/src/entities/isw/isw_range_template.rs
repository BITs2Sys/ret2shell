//! `SeaORM` Entity for an ISW range template (reusable range blueprint).

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, FromJsonQueryResult, IntoActiveModel, QueryOrder, entity::prelude::*,
};
use serde::{Deserialize, Serialize};

/// One VM in a range template's topology.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct VmSpec {
  /// logical name within the range, e.g. "web01" / "dc01".
  pub logical_name: String,
  /// "linux" | "windows".
  pub guest_os: String,
  /// `.vmx` path, relative to the host-agent's range root (agent resolves it).
  pub vmx: String,
  /// key into the host-agent's local guest-credential store.
  pub creds_ref: String,
}

/// VPN blueprint used to broker team access into the isolated range network.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct VpnSpec {
  /// "wireguard" | "openvpn".
  pub kind: String,
  /// server endpoint teams connect to, e.g. "host-a:51820".
  pub server_endpoint: String,
  /// range subnet cidr; may contain a `<group>` placeholder.
  pub subnet: String,
}

/// The complete topology of a range template.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Topology {
  /// isolated host-only vmnet / LAN segment name, e.g. "vmnet12".
  pub vmnet: String,
  pub vms: Vec<VmSpec>,
  pub vpn: Option<VpnSpec>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_range_template")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub game_id: i64,
  pub name: String,
  pub brief: String,
  #[sea_orm(column_type = "JsonBinary")]
  pub topology: Topology,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::super::game::Entity",
    from = "Column::GameId",
    to = "super::super::game::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Game,
}

impl Related<super::super::game::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Game.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn list_by_game<C>(db: &C, game_id: i64) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::GameId.eq(game_id))
    .order_by_asc(Column::Id)
    .all(db)
    .await
}

pub async fn create<C>(db: &C, template: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let template = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..template.into_active_model().reset_all()
  };
  template.insert(db).await
}

pub async fn update<C>(db: &C, template: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let template = ActiveModel {
    id: ActiveValue::Unchanged(template.id),
    created_at: ActiveValue::NotSet,
    game_id: ActiveValue::NotSet,
    ..template.into_active_model().reset_all()
  };
  template.update(db).await
}

pub async fn delete<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
