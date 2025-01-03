use sea_orm_migration::prelude::*;

use super::m_20240104_000001_create_game::Game;
pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20241226_000001_game_traffic"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Game::Table)
          .modify_column(ColumnDef::new(Game::Traffic).text())
          .to_owned(),
      )
      .await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    Ok(())
  }
}
