use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::m_20240101_000005_link_ip_user::User2Ip;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20250721_000001_create_ip_time_info"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(User2Ip::Table)
          .add_column_if_not_exists(
            ColumnDef::new(User2Ip::LastActiveAt)
              .timestamp_with_time_zone()
              .default(CurrentTimestamp),
          )
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(User2Ip::Table)
          .drop_column(User2Ip::LastActiveAt)
          .to_owned(),
      )
      .await?;
    Ok(())
  }
}
