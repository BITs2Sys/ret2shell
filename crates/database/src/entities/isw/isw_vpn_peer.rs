//! `SeaORM` Entity for a per-team VPN peer into an ISW range network.

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "isw_vpn_peer")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub range_id: i64,
  pub team_id: i64,
  /// wireguard/openvpn public key or client identity.
  pub public_key: String,
  /// assigned vpn address inside the range subnet.
  pub address: String,
  /// opaque reference to the generated client config (stored out-of-band).
  pub config_ref: Option<String>,
  pub revoked: bool,
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

pub async fn get_for_team<C>(
  db: &C, range_id: i64, team_id: i64,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::RangeId.eq(range_id))
    .filter(Column::TeamId.eq(team_id))
    .filter(Column::Revoked.eq(false))
    .order_by_desc(Column::CreatedAt)
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

pub async fn create<C>(db: &C, peer: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let peer = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..peer.into_active_model().reset_all()
  };
  peer.insert(db).await
}

pub async fn set_revoked<C>(db: &C, id: i64, revoked: bool) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let peer = ActiveModel {
    id: ActiveValue::Unchanged(id),
    revoked: ActiveValue::Set(revoked),
    ..Default::default()
  };
  peer.update(db).await.map(|_| ())
}
