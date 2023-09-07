use sea_orm_migration::prelude::*;

use super::{m20210101_000002_create_user::User, m20210101_000012_create_ip_address::IpAddress};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20210101_000016_create_user_ip_address_ref"
    }
}

#[derive(Iden)]
pub enum User2IpAddress {
    Table,
    Id,
    UserId,
    IpAddressId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User2IpAddress::Table)
                    .col(
                        ColumnDef::new(User2IpAddress::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User2IpAddress::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("user2ip_address_user_id_fkey")
                            .from(User2IpAddress::Table, User2IpAddress::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(User2IpAddress::IpAddressId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("user2ip_address_ip_address_id_fkey")
                            .from(User2IpAddress::Table, User2IpAddress::IpAddressId)
                            .to(IpAddress::Table, IpAddress::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User2IpAddress::Table).to_owned())
            .await
    }
}
