use sea_orm_migration::prelude::*;

use super::m20221109_000002_create_user::User;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221110_000002_create_media"
    }
}

#[derive(Iden)]
pub enum Media {
    Table,
    Id,
    Name,
    Hash,
    UploaderId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .col(
                        ColumnDef::new(Media::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Media::Name).string_len(255).not_null())
                    .col(
                        ColumnDef::new(Media::Hash)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Media::UploaderId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("media_uploader_id_fkey")
                            .from(Media::Table, Media::UploaderId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Media::Table).to_owned())
            .await
    }
}
