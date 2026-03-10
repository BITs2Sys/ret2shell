use chrono::{
  DateTime, Utc,
  serde::{ts_seconds, ts_seconds_option},
};
use sea_orm::{ActiveValue, IntoActiveModel, QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "game_registry_source")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub name: String,
  pub git_url: String,
  pub branch: String,
  pub enabled: bool,
  pub priority: i32,
  pub publish_enabled: bool,
  pub private_source: bool,
  #[serde(with = "ts_seconds_option", default = "Option::default")]
  pub last_fetched_at: Option<DateTime<Utc>>,
  pub last_error: Option<String>,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  #[serde(with = "ts_seconds")]
  pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get<C>(db: &C, id: i64) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find_by_id(id).one(db).await
}

pub async fn get_list<C>(db: &C) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .order_by_desc(Column::Priority)
    .order_by_asc(Column::Name)
    .all(db)
    .await
}

pub async fn create<C>(db: &C, source: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let source = ActiveModel {
    id: ActiveValue::NotSet,
    ..source.into_active_model().reset_all()
  };
  source.insert(db).await
}

pub async fn update<C>(db: &C, source: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  let source = ActiveModel {
    id: ActiveValue::Unchanged(source.id),
    ..source.into_active_model().reset_all()
  };
  source.update(db).await
}

pub async fn delete<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_by_id(id).exec(db).await.map(|_| ())
}
