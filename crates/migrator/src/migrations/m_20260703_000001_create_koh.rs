use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{
  m_20240104_000003_create_team::Team, m_20240104_000004_create_challenge::Challenge,
  m_20240104_000008_create_extra::Extra,
};

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260703_000001_create_koh"
  }
}

#[derive(Iden)]
pub enum KohIdentifier {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  Identifier,
}

#[derive(Iden)]
pub enum KohState {
  Table,
  ChallengeId,
  CurrentIdentifier,
  CurrentTeamId,
  LastCheckedAt,
  LastAwardedAt,
  LastError,
}

#[derive(Iden)]
pub enum KohEvent {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  PreviousTeamId,
  Identifier,
  Status,
  Message,
  ScoreDelta,
  Tick,
}

#[derive(Iden)]
pub enum KohAward {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  Tick,
  Rank,
  Percent,
  Score,
  ExtraId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(KohIdentifier::Table)
          .col(
            ColumnDef::new(KohIdentifier::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(KohIdentifier::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(
            ColumnDef::new(KohIdentifier::ChallengeId)
              .big_integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(KohIdentifier::Table, KohIdentifier::ChallengeId)
              .to(Challenge::Table, Challenge::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(
            ColumnDef::new(KohIdentifier::TeamId)
              .big_integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(KohIdentifier::Table, KohIdentifier::TeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(
            ColumnDef::new(KohIdentifier::Identifier)
              .string()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(KohIdentifier::Table)
          .col(KohIdentifier::ChallengeId)
          .col(KohIdentifier::TeamId)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(KohIdentifier::Table)
          .col(KohIdentifier::ChallengeId)
          .col(KohIdentifier::Identifier)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(KohState::Table)
          .col(
            ColumnDef::new(KohState::ChallengeId)
              .big_integer()
              .not_null()
              .primary_key(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(KohState::Table, KohState::ChallengeId)
              .to(Challenge::Table, Challenge::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(KohState::CurrentIdentifier).text())
          .col(ColumnDef::new(KohState::CurrentTeamId).big_integer())
          .foreign_key(
            ForeignKey::create()
              .from(KohState::Table, KohState::CurrentTeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .col(ColumnDef::new(KohState::LastCheckedAt).timestamp_with_time_zone())
          .col(ColumnDef::new(KohState::LastAwardedAt).timestamp_with_time_zone())
          .col(ColumnDef::new(KohState::LastError).text())
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(KohEvent::Table)
          .col(
            ColumnDef::new(KohEvent::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(KohEvent::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(
            ColumnDef::new(KohEvent::ChallengeId)
              .big_integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(KohEvent::Table, KohEvent::ChallengeId)
              .to(Challenge::Table, Challenge::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(KohEvent::TeamId).big_integer())
          .foreign_key(
            ForeignKey::create()
              .from(KohEvent::Table, KohEvent::TeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .col(ColumnDef::new(KohEvent::PreviousTeamId).big_integer())
          .foreign_key(
            ForeignKey::create()
              .from(KohEvent::Table, KohEvent::PreviousTeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .col(ColumnDef::new(KohEvent::Identifier).text())
          .col(ColumnDef::new(KohEvent::Status).string().not_null())
          .col(ColumnDef::new(KohEvent::Message).text())
          .col(
            ColumnDef::new(KohEvent::ScoreDelta)
              .integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(KohEvent::Tick).big_integer())
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(KohAward::Table)
          .col(
            ColumnDef::new(KohAward::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(KohAward::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(
            ColumnDef::new(KohAward::ChallengeId)
              .big_integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(KohAward::Table, KohAward::ChallengeId)
              .to(Challenge::Table, Challenge::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(KohAward::TeamId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(KohAward::Table, KohAward::TeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(KohAward::Tick).big_integer().not_null())
          .col(ColumnDef::new(KohAward::Rank).integer())
          .col(ColumnDef::new(KohAward::Percent).integer())
          .col(ColumnDef::new(KohAward::Score).integer().not_null())
          .col(ColumnDef::new(KohAward::ExtraId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(KohAward::Table, KohAward::ExtraId)
              .to(Extra::Table, Extra::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(KohAward::Table)
          .col(KohAward::ChallengeId)
          .col(KohAward::Tick)
          .col(KohAward::TeamId)
          .unique()
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(KohAward::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(KohEvent::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(KohState::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(KohIdentifier::Table).to_owned())
      .await?;
    Ok(())
  }
}
