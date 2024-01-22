use sea_orm_migration::prelude::*;

use super::{m20210101_000007_create_challenge::Challenge, m20210101_000022_create_plan::Plan};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000023_create_plan_challenge_ref"
    }
}

#[derive(Iden)]
pub enum Plan2Challenge {
    Table,
    Id,
    PlanId,
    ChallengeId,
}

#[axum::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Plan2Challenge::Table)
                    .col(
                        ColumnDef::new(Plan2Challenge::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Plan2Challenge::PlanId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan2Challenge::Table, Plan2Challenge::PlanId)
                            .to(Plan::Table, Plan::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Plan2Challenge::ChallengeId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Plan2Challenge::Table, Plan2Challenge::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Plan2Challenge::Table).to_owned())
            .await
    }
}
