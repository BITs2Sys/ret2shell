use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{m20210101_000002_create_user::User, m20210101_000004_create_game::Game};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000020_create_calendar"
    }
}

#[derive(Iden)]
pub enum Calendar {
    Table,
    Id,
    GameId,
    ReporterId,
    Name,
    Intro,
    Link,
    StartTime,
    EndTime,
    Audited,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Calendar::Table)
                    .col(
                        ColumnDef::new(Calendar::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Calendar::Name).string_len(63).not_null())
                    .col(ColumnDef::new(Calendar::Intro).text().not_null())
                    .col(ColumnDef::new(Calendar::Link).string_len(511).not_null())
                    .col(
                        ColumnDef::new(Calendar::StartTime)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Calendar::EndTime)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Calendar::Audited)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Calendar::GameId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("calendar_game_id_fkey")
                            .from(Calendar::Table, Calendar::GameId)
                            .to(Game::Table, Game::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(Calendar::ReporterId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("calendar_reporter_id_fkey")
                            .from(Calendar::Table, Calendar::ReporterId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Calendar::Table).to_owned())
            .await
    }
}
