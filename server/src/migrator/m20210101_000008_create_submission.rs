use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{m20210101_000002_create_user::User, m20210101_000007_create_challenge::Challenge};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000008_create_submission"
    }
}

#[derive(Iden)]
pub enum Submission {
    Table,
    Id,
    CreatedAt,
    UserId,
    ChallengeId,
    Content,
    Solved,
    WithScore,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Submission::Table)
                    .col(
                        ColumnDef::new(Submission::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Submission::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Submission::UserId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_user_id_fkey")
                            .from(Submission::Table, Submission::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Submission::ChallengeId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_challenge_id_fkey")
                            .from(Submission::Table, Submission::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Submission::Content).text().not_null())
                    .col(ColumnDef::new(Submission::Solved).boolean().not_null())
                    .col(ColumnDef::new(Submission::WithScore).boolean().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Submission::Table).to_owned())
            .await
    }
}
