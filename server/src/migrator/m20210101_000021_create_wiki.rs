use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m20210101_000002_create_user::User;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000021_create_wiki"
    }
}

#[derive(Iden)]
pub enum Wiki {
    Table,
    Id,
    Title,
    PublishedAt,
    UpdatedAt,
    AuthorId,
    Content,
    Parent,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Wiki::Table)
                    .col(
                        ColumnDef::new(Wiki::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Wiki::Title).string_len(127).not_null())
                    .col(
                        ColumnDef::new(Wiki::PublishedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Wiki::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(ColumnDef::new(Wiki::AuthorId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("answer_author_id_fkey")
                            .from(Wiki::Table, Wiki::AuthorId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Wiki::Content).text().not_null())
                    .col(ColumnDef::new(Wiki::Parent).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("answer_parent_fkey")
                            .from(Wiki::Table, Wiki::Parent)
                            .to(Wiki::Table, Wiki::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Wiki::Table).to_owned())
            .await
    }
}
