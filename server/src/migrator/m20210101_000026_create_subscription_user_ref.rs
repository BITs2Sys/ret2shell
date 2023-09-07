use sea_orm_migration::prelude::*;

use super::{
    m20210101_000002_create_user::User, m20210101_000025_create_subscription::Subscription,
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000026_create_subscription_user_ref"
    }
}

#[derive(Iden)]
pub enum User2Subscription {
    Table,
    Id,
    UserId,
    SubscriptionId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User2Subscription::Table)
                    .col(
                        ColumnDef::new(User2Subscription::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User2Subscription::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("user_2_subscription_user_id_fkey")
                            .from(User2Subscription::Table, User2Subscription::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(User2Subscription::SubscriptionId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("user_2_subscription_subscription_id_fkey")
                            .from(User2Subscription::Table, User2Subscription::SubscriptionId)
                            .to(Subscription::Table, Subscription::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User2Subscription::Table).to_owned())
            .await
    }
}
