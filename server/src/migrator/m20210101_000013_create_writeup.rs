use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{
    m20210101_000002_create_user::User, m20210101_000004_create_game::Game,
    m20210101_000008_create_team::Team,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000013_create_writeup"
    }
}

// Write up is for game.
#[derive(Iden)]
pub enum WriteUp {
    Table,
    Id,
    Title,
    UpdatedAt,
    PublishedAt,
    AuthorId,
    TeamId,
    GameId,
    Hidden,
    Content,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WriteUp::Table)
                    .col(
                        ColumnDef::new(WriteUp::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WriteUp::Title).string_len(127).not_null())
                    .col(
                        ColumnDef::new(WriteUp::PublishedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(WriteUp::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(WriteUp::AuthorId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(WriteUp::Table, WriteUp::AuthorId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(WriteUp::TeamId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(WriteUp::Table, WriteUp::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(WriteUp::GameId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(WriteUp::Table, WriteUp::GameId)
                            .to(Game::Table, Game::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(WriteUp::Hidden)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(WriteUp::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WriteUp::Table).to_owned())
            .await
    }
}
