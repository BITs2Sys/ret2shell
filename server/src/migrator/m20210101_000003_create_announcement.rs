use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m20210101_000002_create_user::User;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000003_create_announcement"
    }
}

#[derive(Iden)]
pub enum Announcement {
    Table,
    Id,
    Title,
    UpdatedAt,
    PublishedAt,
    PublisherId,
    Content,
    Pinned,
}

#[axum::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Announcement::Table)
                    .col(
                        ColumnDef::new(Announcement::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Announcement::Title)
                            .string_len(127)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Announcement::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Announcement::PublishedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .col(
                        ColumnDef::new(Announcement::PublisherId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Announcement::Table, Announcement::PublisherId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Announcement::Content).text().not_null())
                    .col(ColumnDef::new(Announcement::Pinned).boolean().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Announcement::Table).to_owned())
            .await
    }
}
