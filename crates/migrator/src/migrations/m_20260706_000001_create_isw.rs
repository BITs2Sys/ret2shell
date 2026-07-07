//! BITs2CTF fork: ISW (Internal Security Warfare) range-mode schema.
//!
//! Purely additive: seven new tables with FKs into existing `game`/`team`/`challenge`.
//! Appended at the TAIL of the migrator so it never reorders relative to upstream
//! migrations (the FKs require those parent tables to already exist).

use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{
  m_20240104_000001_create_game::Game, m_20240104_000003_create_team::Team,
  m_20240104_000004_create_challenge::Challenge,
};

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260706_000001_create_isw"
  }
}

#[derive(Iden)]
pub enum IswHost {
  Table,
  Id,
  CreatedAt,
  Name,
  Address,
  ApiPort,
  Os,
  Fingerprint,
  Enabled,
  Status,
  FreeMemMb,
  LastHeartbeat,
}

#[derive(Iden)]
pub enum IswRangeTemplate {
  Table,
  Id,
  CreatedAt,
  GameId,
  Name,
  Brief,
  Topology,
}

#[derive(Iden)]
pub enum IswRange {
  Table,
  Id,
  CreatedAt,
  TemplateId,
  HostId,
  GroupIndex,
  Name,
  Status,
  ArmedAt,
  SnapshotName,
  LastError,
}

#[derive(Iden)]
pub enum IswVm {
  Table,
  Id,
  CreatedAt,
  RangeId,
  LogicalName,
  GuestOs,
  VmxPath,
  Ip,
  PowerState,
  ToolsState,
}

#[derive(Iden)]
pub enum IswFlag {
  Table,
  Id,
  CreatedAt,
  RangeId,
  ChallengeId,
  VmId,
  GuestPath,
  ValueHash,
  Round,
  InjectedAt,
  Verified,
  LastError,
}

#[derive(Iden)]
pub enum IswAssignment {
  Table,
  Id,
  CreatedAt,
  GameId,
  RangeId,
  TeamId,
}

#[derive(Iden)]
pub enum IswVpnPeer {
  Table,
  Id,
  CreatedAt,
  RangeId,
  TeamId,
  PublicKey,
  Address,
  ConfigRef,
  Revoked,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // isw_host
    manager
      .create_table(
        Table::create()
          .table(IswHost::Table)
          .col(
            ColumnDef::new(IswHost::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswHost::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswHost::Name).string().not_null())
          .col(ColumnDef::new(IswHost::Address).string().not_null())
          .col(ColumnDef::new(IswHost::ApiPort).integer().not_null())
          .col(ColumnDef::new(IswHost::Os).string().not_null())
          .col(ColumnDef::new(IswHost::Fingerprint).text())
          .col(
            ColumnDef::new(IswHost::Enabled)
              .boolean()
              .not_null()
              .default(true),
          )
          .col(
            ColumnDef::new(IswHost::Status)
              .string()
              .not_null()
              .default("offline"),
          )
          .col(ColumnDef::new(IswHost::FreeMemMb).big_integer())
          .col(ColumnDef::new(IswHost::LastHeartbeat).timestamp_with_time_zone())
          .to_owned(),
      )
      .await?;

    // isw_range_template
    manager
      .create_table(
        Table::create()
          .table(IswRangeTemplate::Table)
          .col(
            ColumnDef::new(IswRangeTemplate::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswRangeTemplate::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(
            ColumnDef::new(IswRangeTemplate::GameId)
              .big_integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .from(IswRangeTemplate::Table, IswRangeTemplate::GameId)
              .to(Game::Table, Game::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswRangeTemplate::Name).string().not_null())
          .col(
            ColumnDef::new(IswRangeTemplate::Brief)
              .text()
              .not_null()
              .default(""),
          )
          .col(
            ColumnDef::new(IswRangeTemplate::Topology)
              .json_binary()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    // isw_range
    manager
      .create_table(
        Table::create()
          .table(IswRange::Table)
          .col(
            ColumnDef::new(IswRange::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswRange::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswRange::TemplateId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswRange::Table, IswRange::TemplateId)
              .to(IswRangeTemplate::Table, IswRangeTemplate::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswRange::HostId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswRange::Table, IswRange::HostId)
              .to(IswHost::Table, IswHost::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Restrict),
          )
          .col(
            ColumnDef::new(IswRange::GroupIndex)
              .integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(IswRange::Name).string().not_null())
          .col(
            ColumnDef::new(IswRange::Status)
              .string()
              .not_null()
              .default("pending"),
          )
          .col(ColumnDef::new(IswRange::ArmedAt).timestamp_with_time_zone())
          .col(ColumnDef::new(IswRange::SnapshotName).string())
          .col(ColumnDef::new(IswRange::LastError).text())
          .to_owned(),
      )
      .await?;

    // isw_vm
    manager
      .create_table(
        Table::create()
          .table(IswVm::Table)
          .col(
            ColumnDef::new(IswVm::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswVm::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswVm::RangeId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswVm::Table, IswVm::RangeId)
              .to(IswRange::Table, IswRange::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswVm::LogicalName).string().not_null())
          .col(ColumnDef::new(IswVm::GuestOs).string().not_null())
          .col(ColumnDef::new(IswVm::VmxPath).text().not_null())
          .col(ColumnDef::new(IswVm::Ip).string())
          .col(
            ColumnDef::new(IswVm::PowerState)
              .string()
              .not_null()
              .default("unknown"),
          )
          .col(
            ColumnDef::new(IswVm::ToolsState)
              .string()
              .not_null()
              .default("unknown"),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(IswVm::Table)
          .col(IswVm::RangeId)
          .col(IswVm::LogicalName)
          .unique()
          .to_owned(),
      )
      .await?;

    // isw_flag
    manager
      .create_table(
        Table::create()
          .table(IswFlag::Table)
          .col(
            ColumnDef::new(IswFlag::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswFlag::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswFlag::RangeId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswFlag::Table, IswFlag::RangeId)
              .to(IswRange::Table, IswRange::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswFlag::ChallengeId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswFlag::Table, IswFlag::ChallengeId)
              .to(Challenge::Table, Challenge::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswFlag::VmId).big_integer())
          .foreign_key(
            ForeignKey::create()
              .from(IswFlag::Table, IswFlag::VmId)
              .to(IswVm::Table, IswVm::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .col(ColumnDef::new(IswFlag::GuestPath).text().not_null())
          .col(ColumnDef::new(IswFlag::ValueHash).string().not_null())
          .col(
            ColumnDef::new(IswFlag::Round)
              .integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(IswFlag::InjectedAt).timestamp_with_time_zone())
          .col(
            ColumnDef::new(IswFlag::Verified)
              .boolean()
              .not_null()
              .default(false),
          )
          .col(ColumnDef::new(IswFlag::LastError).text())
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(IswFlag::Table)
          .col(IswFlag::RangeId)
          .col(IswFlag::ChallengeId)
          .col(IswFlag::Round)
          .unique()
          .to_owned(),
      )
      .await?;

    // isw_assignment
    manager
      .create_table(
        Table::create()
          .table(IswAssignment::Table)
          .col(
            ColumnDef::new(IswAssignment::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswAssignment::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswAssignment::GameId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswAssignment::Table, IswAssignment::GameId)
              .to(Game::Table, Game::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswAssignment::RangeId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswAssignment::Table, IswAssignment::RangeId)
              .to(IswRange::Table, IswRange::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswAssignment::TeamId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswAssignment::Table, IswAssignment::TeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(IswAssignment::Table)
          .col(IswAssignment::GameId)
          .col(IswAssignment::TeamId)
          .unique()
          .to_owned(),
      )
      .await?;

    // isw_vpn_peer
    manager
      .create_table(
        Table::create()
          .table(IswVpnPeer::Table)
          .col(
            ColumnDef::new(IswVpnPeer::Id)
              .big_integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(
            ColumnDef::new(IswVpnPeer::CreatedAt)
              .timestamp_with_time_zone()
              .not_null()
              .default(CurrentTimestamp),
          )
          .col(ColumnDef::new(IswVpnPeer::RangeId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswVpnPeer::Table, IswVpnPeer::RangeId)
              .to(IswRange::Table, IswRange::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswVpnPeer::TeamId).big_integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .from(IswVpnPeer::Table, IswVpnPeer::TeamId)
              .to(Team::Table, Team::Id)
              .on_update(ForeignKeyAction::Cascade)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .col(ColumnDef::new(IswVpnPeer::PublicKey).text().not_null())
          .col(ColumnDef::new(IswVpnPeer::Address).string().not_null())
          .col(ColumnDef::new(IswVpnPeer::ConfigRef).text())
          .col(
            ColumnDef::new(IswVpnPeer::Revoked)
              .boolean()
              .not_null()
              .default(false),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(IswVpnPeer::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswAssignment::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswFlag::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswVm::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswRange::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswRangeTemplate::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(IswHost::Table).to_owned())
      .await?;
    Ok(())
  }
}
