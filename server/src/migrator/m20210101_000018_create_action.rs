use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m20210101_000007_create_challenge::Challenge;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000018_create_action"
    }
}

#[derive(Iden)]
pub enum Action {
    Table,
    Id,
    ChallengeId,
    CreatedAt,
    StartedAt,
    CommitId,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Action::Table)
                    .col(
                        ColumnDef::new(Action::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Action::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Action::StartedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Action::ChallengeId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Action::Table, Action::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Action::Status).small_integer().not_null())
                    .col(ColumnDef::new(Action::CommitId).string_len(63).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Action::Table).to_owned())
            .await
    }
}
