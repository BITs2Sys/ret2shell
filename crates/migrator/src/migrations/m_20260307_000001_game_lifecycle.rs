use sea_orm_migration::prelude::*;

use super::m_20240104_000001_create_game::Game;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260307_000001_game_lifecycle"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Game::Table)
          .add_column_if_not_exists(ColumnDef::new(Game::Lifecycle).text())
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
          .drop_column(Game::Lifecycle)
          .to_owned(),
      )
      .await?;
    Ok(())
  }
}
