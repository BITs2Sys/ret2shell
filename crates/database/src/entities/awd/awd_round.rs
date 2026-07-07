//! `SeaORM` Entity: per-team, per-round AWD flag + SLA check result. Only the flag
//! sha256 is stored (plaintext lives in the machine).

use chrono::{DateTime, Utc, serde::ts_seconds};
use sea_orm::{
  ActiveValue, IntoActiveModel, entity::prelude::*, sea_query::OnConflict,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "awd_round")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  #[serde(with = "ts_seconds")]
  pub created_at: DateTime<Utc>,
  pub challenge_id: i64,
  pub team_id: i64,
  pub round: i64,
  /// sha256 hex of the flag injected into this team's machine this round.
  pub value_hash: String,
  /// SLA service check result for this team's machine this round.
  pub sla_ok: bool,
  /// whether this round's SLA/defense scoring has been finalized (idempotency guard).
  #[serde(default)]
  pub finalized: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// All teams' round rows for a given (challenge, round) — the attack surface.
pub async fn list_by_round<C>(
  db: &C, challenge_id: i64, round: i64,
) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Round.eq(round))
    .all(db)
    .await
}

/// Find the round row whose flag hash matches a submitted flag (the victim).
pub async fn find_by_hash<C>(
  db: &C, challenge_id: i64, round: i64, value_hash: &str,
) -> Result<Option<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Round.eq(round))
    .filter(Column::ValueHash.eq(value_hash))
    .one(db)
    .await
}

pub async fn create<C>(db: &C, model: Model) -> Result<Model, DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(Utc::now()),
    ..model.into_active_model().reset_all()
  }
  .insert(db)
  .await
}

/// Insert (or update the flag/SLA of) a team's round row idempotently on the unique
/// `(challenge, team, round)` key, so a retried round tick never trips the unique
/// index. `finalized` is left untouched on conflict (a current round is never
/// finalized yet, and re-inserting must not un-finalize a completed one).
pub async fn upsert<C>(
  db: &C, challenge_id: i64, team_id: i64, round: i64, value_hash: &str, sla_ok: bool,
  now: DateTime<Utc>,
) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  let model = ActiveModel {
    id: ActiveValue::NotSet,
    created_at: ActiveValue::Set(now),
    challenge_id: ActiveValue::Set(challenge_id),
    team_id: ActiveValue::Set(team_id),
    round: ActiveValue::Set(round),
    value_hash: ActiveValue::Set(value_hash.to_owned()),
    sla_ok: ActiveValue::Set(sla_ok),
    finalized: ActiveValue::Set(false),
  };
  Entity::insert(model)
    .on_conflict(
      OnConflict::columns([Column::ChallengeId, Column::TeamId, Column::Round])
        .update_columns([Column::ValueHash, Column::SlaOk])
        .to_owned(),
    )
    .exec_without_returning(db)
    .await
    .map(|_| ())
}

/// The still-unfinalized round rows for a (challenge, round) — the ones whose SLA and
/// defense scoring has not yet been awarded.
pub async fn list_unfinalized_by_round<C>(
  db: &C, challenge_id: i64, round: i64,
) -> Result<Vec<Model>, DbErr>
where
  C: ConnectionTrait, {
  Entity::find()
    .filter(Column::ChallengeId.eq(challenge_id))
    .filter(Column::Round.eq(round))
    .filter(Column::Finalized.eq(false))
    .all(db)
    .await
}

/// Mark a round row finalized so its SLA/defense scoring is never re-awarded.
pub async fn mark_finalized<C>(db: &C, id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  ActiveModel {
    id: ActiveValue::Unchanged(id),
    finalized: ActiveValue::Set(true),
    ..Default::default()
  }
  .update(db)
  .await
  .map(|_| ())
}

pub async fn delete_by_challenge<C>(db: &C, challenge_id: i64) -> Result<(), DbErr>
where
  C: ConnectionTrait, {
  Entity::delete_many()
    .filter(Column::ChallengeId.eq(challenge_id))
    .exec(db)
    .await
    .map(|_| ())
}
