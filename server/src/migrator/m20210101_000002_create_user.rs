use sea_orm_migration::prelude::*;

use super::m20210101_000001_create_institute::Institute;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000002_create_user"
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Name,
    Password,
    Email,
    Intro,
    CoverPath,
    InstituteId,
    InstituteInfo,
    Permissions,
    Hidden,
    Banned,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Name)
                            .string_len(63)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::Password).string_len(127).not_null())
                    .col(
                        ColumnDef::new(User::Email)
                            .string_len(127)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(User::Intro).text())
                    .col(ColumnDef::new(User::CoverPath).string_len(511))
                    .col(ColumnDef::new(User::InstituteId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(User::Table, User::InstituteId)
                            .to(Institute::Table, Institute::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(ColumnDef::new(User::InstituteInfo).text())
                    .col(
                        ColumnDef::new(User::Permissions)
                            .json_binary()
                            .not_null()
                            .default("[0]"),
                    )
                    .col(
                        ColumnDef::new(User::Hidden)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(User::Banned)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
