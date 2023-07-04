use sea_orm_migration::prelude::*;

use super::{m20221109_000005_create_user::User, m20221110_000001_create_team::Team};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221110_000007_create_user_team_ref"
    }
}

#[derive(Iden)]
pub enum User2Team {
    Table,
    Id,
    UserId,
    TeamId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User2Team::Table)
                    .col(
                        ColumnDef::new(User2Team::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User2Team::UserId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("user2team_user_id_fkey")
                            .from(User2Team::Table, User2Team::UserId)
                            .to(User::Table, User::Id),
                    )
                    .col(ColumnDef::new(User2Team::TeamId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("user2team_team_id_fkey")
                            .from(User2Team::Table, User2Team::TeamId)
                            .to(Team::Table, Team::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User2Team::Table).to_owned())
            .await
    }
}
