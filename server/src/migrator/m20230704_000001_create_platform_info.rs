use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20230704_000001_create_platform_info"
    }
}

#[derive(Iden)]
pub enum PlatformInfo {
    Table,
    Id,
    Captcha,
    Email,
    Media,
    Platform,
    Pusher,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlatformInfo::Table)
                    .col(
                        ColumnDef::new(PlatformInfo::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlatformInfo::Platform).text())
                    .col(ColumnDef::new(PlatformInfo::Captcha).text())
                    .col(ColumnDef::new(PlatformInfo::Email).text())
                    .col(ColumnDef::new(PlatformInfo::Media).text())
                    .col(ColumnDef::new(PlatformInfo::Pusher).text())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlatformInfo::Table).to_owned())
            .await
    }
}
