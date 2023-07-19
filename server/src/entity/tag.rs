use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::challenge::Entity")]
    Challenge,
}

impl Related<super::challenge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Challenge.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
