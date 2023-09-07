use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m20210101_000001_create_institute::Institute;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000004_create_game"
    }
}

#[derive(Iden)]
pub enum Game {
    Table,
    Id,
    UpdatedAt,
    Name,
    Brief,
    Introduction,
    StartTime,
    EndTime,
    RegisterTime,
    ArchiveTime,
    Hidden,
    Frozen,
    HostAsGame,
    TeamSizeLimit,
    CoverPath,
    EnableTeamAudit,
    CanRegisterAfterStarted,
    InstituteId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Game::Table)
                    .col(
                        ColumnDef::new(Game::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Game::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Game::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Game::Brief).string_len(255).not_null())
                    .col(ColumnDef::new(Game::Introduction).text().not_null())
                    .col(
                        ColumnDef::new(Game::StartTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Game::EndTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Game::RegisterTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Game::ArchiveTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Game::Hidden).boolean().not_null())
                    .col(ColumnDef::new(Game::Frozen).boolean().not_null())
                    .col(ColumnDef::new(Game::HostAsGame).boolean().not_null())
                    .col(ColumnDef::new(Game::TeamSizeLimit).integer().not_null())
                    .col(ColumnDef::new(Game::CoverPath).string_len(128).not_null())
                    .col(ColumnDef::new(Game::EnableTeamAudit).boolean().not_null())
                    .col(
                        ColumnDef::new(Game::CanRegisterAfterStarted)
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Game::InstituteId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("game_institute_id_fkey")
                            .from(Game::Table, Game::InstituteId)
                            .to(Institute::Table, Institute::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Game::Table).to_owned())
            .await
    }
}
