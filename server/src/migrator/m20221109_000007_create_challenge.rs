use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{m20221109_000004_create_game::Game, m20221109_000006_create_tag::Tag};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221109_000007_create_challenge"
    }
}

#[derive(Iden)]
pub enum Challenge {
    Table,
    Id,
    UpdatedAt,
    Name,
    Content,
    Hidden,
    GameId,
    TagId,
    InitialScore,
    CurrentScore,
    MinimumScore,
    Decay,
    Bucket,
    Checker,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Challenge::Table)
                    .col(
                        ColumnDef::new(Challenge::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Challenge::Name).string_len(127).not_null())
                    .col(ColumnDef::new(Challenge::Content).text().not_null())
                    .col(ColumnDef::new(Challenge::Hidden).boolean().not_null())
                    .col(ColumnDef::new(Challenge::GameId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("challenge_game_id_fkey")
                            .from(Challenge::Table, Challenge::GameId)
                            .to(Game::Table, Game::Id),
                    )
                    .col(ColumnDef::new(Challenge::TagId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("challenge_tag_id_fkey")
                            .from(Challenge::Table, Challenge::TagId)
                            .to(Tag::Table, Tag::Id),
                    )
                    .col(ColumnDef::new(Challenge::InitialScore).integer().not_null())
                    .col(
                        ColumnDef::new(Challenge::CurrentScore)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Challenge::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Challenge::MinimumScore).integer().not_null())
                    .col(ColumnDef::new(Challenge::Decay).integer().not_null())
                    .col(ColumnDef::new(Challenge::Bucket).string_len(127).not_null())
                    .col(
                        ColumnDef::new(Challenge::Checker)
                            .string_len(127)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Challenge::Table).to_owned())
            .await
    }
}
