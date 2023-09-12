use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000022_create_plan"
    }
}

#[derive(Iden)]
pub enum Plan {
    Table,
    Id,
    Name,
    Price,
    Duration,
    Description,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Plan::Table)
                    .col(
                        ColumnDef::new(Plan::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Plan::Name).string_len(127).not_null())
                    .col(ColumnDef::new(Plan::Price).big_integer().not_null())
                    // timestamp
                    .col(ColumnDef::new(Plan::Duration).big_integer().not_null())
                    .col(ColumnDef::new(Plan::Description).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Plan::Table).to_owned())
            .await
    }
}
