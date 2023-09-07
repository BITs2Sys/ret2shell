use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{m20210101_000002_create_user::User, m20210101_000007_create_challenge::Challenge};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000014_create_answer"
    }
}

// Answer is for challenge.
#[derive(Iden)]
pub enum Answer {
    Table,
    Id,
    Title,
    PublishedAt,
    UpdatedAt,
    AuthorId,
    ChallengeId,
    Content,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Answer::Table)
                    .col(
                        ColumnDef::new(Answer::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Answer::Title).string_len(127).not_null())
                    .col(
                        ColumnDef::new(Answer::PublishedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Answer::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Answer::AuthorId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("answer_author_id_fkey")
                            .from(Answer::Table, Answer::AuthorId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(Answer::ChallengeId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("answer_challenge_id_fkey")
                            .from(Answer::Table, Answer::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Answer::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Answer::Table).to_owned())
            .await
    }
}
