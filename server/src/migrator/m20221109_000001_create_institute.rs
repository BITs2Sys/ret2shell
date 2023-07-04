use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221109_000001_create_institute"
    }
}

#[derive(Iden)]
pub enum Institute {
    Table,
    Id,
    Name,
    ViaEmail,
    EmailDomain,
    ViaCas,
    CasIden,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Institute::Table)
                    .col(
                        ColumnDef::new(Institute::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Institute::Name).string_len(127).not_null())
                    .col(ColumnDef::new(Institute::ViaEmail).boolean().not_null())
                    .col(
                        ColumnDef::new(Institute::EmailDomain)
                            .string_len(127)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Institute::ViaCas).boolean().not_null())
                    .col(
                        ColumnDef::new(Institute::CasIden)
                            .string_len(127)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Institute::Table).to_owned())
            .await
    }
}
