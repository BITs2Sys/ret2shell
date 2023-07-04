use sea_orm_migration::prelude::*;

use super::m20221109_000007_create_challenge::Challenge;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20221109_000009_create_hint"
    }
}

#[derive(Iden)]
pub enum Hint {
    Table,
    Id,
    ChallengeId,
    Content,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Hint::Table)
                    .col(
                        ColumnDef::new(Hint::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Hint::ChallengeId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_challenge_id_fkey")
                            .from(Hint::Table, Hint::ChallengeId)
                            .to(Challenge::Table, Challenge::Id),
                    )
                    .col(ColumnDef::new(Hint::Content).text().not_null())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Bakery table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hint::Table).to_owned())
            .await
    }
}
