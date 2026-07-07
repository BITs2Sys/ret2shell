//! BITs2CTF fork: AWD (Attack-and-Defense) + AWDP schema. Purely additive: 7 new
//! tables with FKs into existing `challenge`/`team`/`extra`. Appended at the migrator
//! tail.

use sea_orm_migration::prelude::*;
use sea_query::Keyword::CurrentTimestamp;

use super::{
  m_20240104_000003_create_team::Team, m_20240104_000004_create_challenge::Challenge,
  m_20240104_000008_create_extra::Extra,
};

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m_20260707_000001_create_awd"
  }
}

#[derive(Iden)]
enum AwdpState {
  Table,
  ChallengeId,
  LastRound,
  LastCheckedAt,
  LastError,
}

#[derive(Iden)]
enum AwdpSolve {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  SolvedAt,
}

#[derive(Iden)]
enum AwdpAward {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  Round,
  Score,
  ExtraId,
}

#[derive(Iden)]
enum AwdInstance {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  PodName,
  Address,
  Status,
}

#[derive(Iden)]
enum AwdRound {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  TeamId,
  Round,
  ValueHash,
  SlaOk,
  /// whether this round's SLA/defense scoring has already been finalized (makes
  /// round_tick finalization idempotent under retries).
  Finalized,
}

#[derive(Iden)]
enum AwdSteal {
  Table,
  Id,
  CreatedAt,
  ChallengeId,
  Round,
  AttackerTeamId,
  VictimTeamId,
  Score,
  ExtraId,
}

#[derive(Iden)]
enum AwdState {
  Table,
  ChallengeId,
  LastRound,
  LastCheckedAt,
  LastError,
}

fn id_col<T: 'static + Iden>(c: T) -> ColumnDef {
  ColumnDef::new(c)
    .big_integer()
    .not_null()
    .auto_increment()
    .primary_key()
    .to_owned()
}

fn created_col<T: 'static + Iden>(c: T) -> ColumnDef {
  ColumnDef::new(c)
    .timestamp_with_time_zone()
    .not_null()
    .default(CurrentTimestamp)
    .to_owned()
}

fn fk<A, B>(name: &str, from_t: A, from_c: A, to_t: B, to_c: B, on_delete: ForeignKeyAction) -> ForeignKeyCreateStatement
where
  A: 'static + Iden,
  B: 'static + Iden,
{
  ForeignKey::create()
    .name(name)
    .from(from_t, from_c)
    .to(to_t, to_c)
    .on_update(ForeignKeyAction::Cascade)
    .on_delete(on_delete)
    .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // awdp_state
    manager
      .create_table(
        Table::create()
          .table(AwdpState::Table)
          .col(
            ColumnDef::new(AwdpState::ChallengeId)
              .big_integer()
              .not_null()
              .primary_key(),
          )
          .foreign_key(&mut fk(
            "fk_awdp_state_challenge",
            AwdpState::Table,
            AwdpState::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(
            ColumnDef::new(AwdpState::LastRound)
              .big_integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(AwdpState::LastCheckedAt).timestamp_with_time_zone())
          .col(ColumnDef::new(AwdpState::LastError).text())
          .to_owned(),
      )
      .await?;

    // awdp_solve
    manager
      .create_table(
        Table::create()
          .table(AwdpSolve::Table)
          .col(id_col(AwdpSolve::Id))
          .col(created_col(AwdpSolve::CreatedAt))
          .col(ColumnDef::new(AwdpSolve::ChallengeId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awdp_solve_challenge",
            AwdpSolve::Table,
            AwdpSolve::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdpSolve::TeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awdp_solve_team",
            AwdpSolve::Table,
            AwdpSolve::TeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(
            ColumnDef::new(AwdpSolve::SolvedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(AwdpSolve::Table)
          .col(AwdpSolve::ChallengeId)
          .col(AwdpSolve::TeamId)
          .unique()
          .to_owned(),
      )
      .await?;

    // awdp_award
    manager
      .create_table(
        Table::create()
          .table(AwdpAward::Table)
          .col(id_col(AwdpAward::Id))
          .col(created_col(AwdpAward::CreatedAt))
          .col(ColumnDef::new(AwdpAward::ChallengeId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awdp_award_challenge",
            AwdpAward::Table,
            AwdpAward::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdpAward::TeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awdp_award_team",
            AwdpAward::Table,
            AwdpAward::TeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdpAward::Round).big_integer().not_null())
          .col(ColumnDef::new(AwdpAward::Score).integer().not_null())
          .col(ColumnDef::new(AwdpAward::ExtraId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awdp_award_extra",
            AwdpAward::Table,
            AwdpAward::ExtraId,
            Extra::Table,
            Extra::Id,
            ForeignKeyAction::Cascade,
          ))
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(AwdpAward::Table)
          .col(AwdpAward::ChallengeId)
          .col(AwdpAward::TeamId)
          .col(AwdpAward::Round)
          .unique()
          .to_owned(),
      )
      .await?;

    // awd_instance
    manager
      .create_table(
        Table::create()
          .table(AwdInstance::Table)
          .col(id_col(AwdInstance::Id))
          .col(created_col(AwdInstance::CreatedAt))
          .col(ColumnDef::new(AwdInstance::ChallengeId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_instance_challenge",
            AwdInstance::Table,
            AwdInstance::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdInstance::TeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_instance_team",
            AwdInstance::Table,
            AwdInstance::TeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdInstance::PodName).string().not_null())
          .col(ColumnDef::new(AwdInstance::Address).string())
          .col(
            ColumnDef::new(AwdInstance::Status)
              .string()
              .not_null()
              .default("pending"),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(AwdInstance::Table)
          .col(AwdInstance::ChallengeId)
          .col(AwdInstance::TeamId)
          .unique()
          .to_owned(),
      )
      .await?;

    // awd_round
    manager
      .create_table(
        Table::create()
          .table(AwdRound::Table)
          .col(id_col(AwdRound::Id))
          .col(created_col(AwdRound::CreatedAt))
          .col(ColumnDef::new(AwdRound::ChallengeId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_round_challenge",
            AwdRound::Table,
            AwdRound::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdRound::TeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_round_team",
            AwdRound::Table,
            AwdRound::TeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdRound::Round).big_integer().not_null())
          .col(ColumnDef::new(AwdRound::ValueHash).string().not_null())
          .col(
            ColumnDef::new(AwdRound::SlaOk)
              .boolean()
              .not_null()
              .default(false),
          )
          .col(
            ColumnDef::new(AwdRound::Finalized)
              .boolean()
              .not_null()
              .default(false),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(AwdRound::Table)
          .col(AwdRound::ChallengeId)
          .col(AwdRound::TeamId)
          .col(AwdRound::Round)
          .unique()
          .to_owned(),
      )
      .await?;
    // supports verify_attack's find_by_hash (challenge, round, value_hash) lookup on
    // every attack submission.
    manager
      .create_index(
        Index::create()
          .table(AwdRound::Table)
          .col(AwdRound::ChallengeId)
          .col(AwdRound::Round)
          .col(AwdRound::ValueHash)
          .to_owned(),
      )
      .await?;

    // awd_steal
    manager
      .create_table(
        Table::create()
          .table(AwdSteal::Table)
          .col(id_col(AwdSteal::Id))
          .col(created_col(AwdSteal::CreatedAt))
          .col(ColumnDef::new(AwdSteal::ChallengeId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_steal_challenge",
            AwdSteal::Table,
            AwdSteal::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdSteal::Round).big_integer().not_null())
          .col(ColumnDef::new(AwdSteal::AttackerTeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_steal_attacker",
            AwdSteal::Table,
            AwdSteal::AttackerTeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdSteal::VictimTeamId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_steal_victim",
            AwdSteal::Table,
            AwdSteal::VictimTeamId,
            Team::Table,
            Team::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(ColumnDef::new(AwdSteal::Score).integer().not_null())
          .col(ColumnDef::new(AwdSteal::ExtraId).big_integer().not_null())
          .foreign_key(&mut fk(
            "fk_awd_steal_extra",
            AwdSteal::Table,
            AwdSteal::ExtraId,
            Extra::Table,
            Extra::Id,
            ForeignKeyAction::Cascade,
          ))
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(AwdSteal::Table)
          .col(AwdSteal::ChallengeId)
          .col(AwdSteal::Round)
          .col(AwdSteal::AttackerTeamId)
          .col(AwdSteal::VictimTeamId)
          .unique()
          .to_owned(),
      )
      .await?;

    // awd_state
    manager
      .create_table(
        Table::create()
          .table(AwdState::Table)
          .col(
            ColumnDef::new(AwdState::ChallengeId)
              .big_integer()
              .not_null()
              .primary_key(),
          )
          .foreign_key(&mut fk(
            "fk_awd_state_challenge",
            AwdState::Table,
            AwdState::ChallengeId,
            Challenge::Table,
            Challenge::Id,
            ForeignKeyAction::Cascade,
          ))
          .col(
            ColumnDef::new(AwdState::LastRound)
              .big_integer()
              .not_null()
              .default(0),
          )
          .col(ColumnDef::new(AwdState::LastCheckedAt).timestamp_with_time_zone())
          .col(ColumnDef::new(AwdState::LastError).text())
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    for t in [
      AwdSteal::Table.into_iden(),
      AwdRound::Table.into_iden(),
      AwdInstance::Table.into_iden(),
      AwdState::Table.into_iden(),
      AwdpAward::Table.into_iden(),
      AwdpSolve::Table.into_iden(),
      AwdpState::Table.into_iden(),
    ] {
      manager.drop_table(Table::drop().table(t).to_owned()).await?;
    }
    Ok(())
  }
}
