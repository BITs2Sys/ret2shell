use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m20221109_000004_create_game::Game;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221109_000005_create_notification"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notification::Table)
                    .col(
                        ColumnDef::new(Notification::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Notification::Title)
                            .string_len(127)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Notification::Content).text().not_null())
                    .col(
                        ColumnDef::new(Notification::PublishedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Notification::GameId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("notification_game_id_fkey")
                            .from(Notification::Table, Notification::GameId)
                            .to(Game::Table, Game::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notification::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Notification {
    Table,
    Id,
    Title,
    Content,
    PublishedAt,
    GameId,
}
