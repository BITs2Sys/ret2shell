use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{m20221109_000002_create_user::User, m20221109_000004_create_game::Game};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221110_000004_create_writeup"
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
    GameId,
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
                            .name("writeup_author_id_fkey")
                            .from(WriteUp::Table, WriteUp::AuthorId)
                            .to(User::Table, User::Id),
                    )
                    .col(ColumnDef::new(WriteUp::GameId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("writeup_game_id_fkey")
                            .from(WriteUp::Table, WriteUp::GameId)
                            .to(Game::Table, Game::Id),
                    )
                    .col(ColumnDef::new(WriteUp::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WriteUp::Table).to_owned())
            .await
    }
}
