use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "platform_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub platform: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub captcha: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub media: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pusher: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
