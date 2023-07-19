use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "team")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub game_id: i64,
    pub token: Option<String>,
    pub state: i32,
    pub institute_id: Option<i64>,
    pub score: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub history: Option<Json>,
    pub last_active_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::game::Entity",
        from = "Column::GameId",
        to = "super::game::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Game,
    #[sea_orm(
        belongs_to = "super::institute::Entity",
        from = "Column::InstituteId",
        to = "super::institute::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Institute,
    #[sea_orm(has_many = "super::user2_team::Entity")]
    User2Team,
}

impl Related<super::game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Game.def()
    }
}

impl Related<super::institute::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Institute.def()
    }
}

impl Related<super::user2_team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User2Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
