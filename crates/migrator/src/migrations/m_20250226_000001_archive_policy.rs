use sea_orm_migration::prelude::*;

use super::m_20240104_000001_create_game::Game;
pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20250226_000001_archive_policy"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Game::Table)
          .add_column_if_not_exists(
            ColumnDef::new(Game::ArchivePolicy).json_binary())
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Game::Table)
          .drop_column(Game::ArchivePolicy)
          .to_owned(),
      )
      .await?;
    Ok(())
  }
}
