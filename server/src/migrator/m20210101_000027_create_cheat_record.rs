use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{
    m20210101_000002_create_user::User, m20210101_000007_create_challenge::Challenge,
    m20210101_000008_create_team::Team,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000027_create_cheat_record"
    }
}

#[derive(Iden)]
pub enum CheatRecord {
    Table,
    Id,
    CreatedAt,
    Reason,
    ChallengeId,
    UserId,
    TeamId,
}

#[axum::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CheatRecord::Table)
                    .col(
                        ColumnDef::new(CheatRecord::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CheatRecord::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(CheatRecord::Reason).text().not_null())
                    .col(
                        ColumnDef::new(CheatRecord::ChallengeId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CheatRecord::Table, CheatRecord::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(CheatRecord::UserId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(CheatRecord::Table, CheatRecord::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(CheatRecord::TeamId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(CheatRecord::Table, CheatRecord::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CheatRecord::Table).to_owned())
            .await
    }
}
