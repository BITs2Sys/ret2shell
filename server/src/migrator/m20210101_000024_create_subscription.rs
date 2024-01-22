use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000024_create_subscription"
    }
}

#[derive(Iden)]
pub enum Subscription {
    Table,
    Id,
    PlanId,
    SubscriberId,
    StartTime,
}

#[axum::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscription::Table)
                    .col(
                        ColumnDef::new(Subscription::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Subscription::PlanId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscription::SubscriberId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscription::StartTime)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(CurrentTimestamp),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscription::Table).to_owned())
            .await
    }
}
