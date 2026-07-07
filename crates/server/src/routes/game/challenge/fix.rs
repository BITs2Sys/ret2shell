use axum::{
  Extension, Json, Router,
  extract::{DefaultBodyLimit, Multipart, State},
  response::IntoResponse,
  routing::{get, patch, post},
};
use futures::TryStreamExt;
use nanoid::nanoid;
use r2s_bucket::Bucket;
use r2s_config::cluster::FixConfig;
use r2s_database::{challenge, game, submission, team, user::Permission};
use r2s_migrator::Database;
use r2s_queue::Queue;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio_util::io::StreamReader;
use tower_http::request_id::RequestId;
use tracing::{info, warn};

use crate::{
  middleware::{
    auth::{Token, is_game_admin},
    data::extract_team,
  },
  traits::{GlobalState, ResponseError},
  utility::fix::{
    FixSubmissionMeta, encode_fix_submission_meta, fix_upload_dir, fix_upload_path,
    is_fix_submission,
  },
};

const LABEL_ALPHABET: [char; 62] = [
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
  'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
  'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
  'V', 'W', 'X', 'Y', 'Z',
];

#[derive(Serialize)]
pub(super) struct FixStatus {
  pub config: Option<FixConfig>,
  pub attempts_used: u64,
  pub attempts_remaining: Option<u64>,
  pub solved: bool,
}

// BITs2CTF fork (B2): count over the UNION of this user's and this team's fix
// submissions. Previously the accounting bucket flipped between team and user
// depending on game state, letting a player double their attempts by straddling
// the two. Generic over the connection so it can run inside the submit txn.
async fn count_fix_attempts<C>(
  db: &C, game: &game::Model, challenge: &challenge::Model, team: Option<&team::Model>,
  user_id: i64,
) -> Result<u64, ResponseError>
where
  C: ConnectionTrait, {
  let mut ids = std::collections::HashSet::new();
  for submission in submission::get_list(
    db,
    false,
    true,
    Some(game.id),
    Some(challenge.id),
    None,
    Some(user_id),
    true,
  )
  .await?
  {
    if is_fix_submission(submission.content.as_deref()) {
      ids.insert(submission.id);
    }
  }
  if let Some(team) = team {
    for submission in submission::get_list(
      db,
      false,
      true,
      Some(game.id),
      Some(challenge.id),
      Some(team.id),
      None,
      false,
    )
    .await?
    {
      if is_fix_submission(submission.content.as_deref()) {
        ids.insert(submission.id);
      }
    }
  }
  Ok(ids.len() as u64)
}

async fn solved<C>(
  db: &C, game: &game::Model, challenge: &challenge::Model, team: Option<&team::Model>,
  user_id: i64,
) -> Result<bool, ResponseError>
where
  C: ConnectionTrait, {
  if submission::count(
    db,
    true,
    Some(game.id),
    Some(challenge.id),
    None,
    Some(user_id),
    None,
    true,
  )
  .await?
    > 0
  {
    return Ok(true);
  }
  if let Some(team) = team
    && submission::count(
      db,
      true,
      Some(game.id),
      Some(challenge.id),
      Some(team.id),
      None,
      None,
      false,
    )
    .await?
      > 0
  {
    return Ok(true);
  }
  Ok(false)
}

/// BITs2CTF fork (A5): Fix admin routes, merged above the `game_admin_required`
/// layer by the challenge router.
pub(crate) fn admin_router() -> Router<GlobalState> {
  Router::new().route("/fix", patch(update_fix_config).delete(delete_fix_config))
}

/// Fix player routes (view status + submit artifact), merged below the admin layer.
pub(crate) fn player_router() -> Router<GlobalState> {
  Router::new()
    .route("/fix", get(get_fix_config))
    .route(
      "/fix/submit",
      post(submit_fix).route_layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
    )
}

pub(crate) fn validate_fix_config(config: &FixConfig) -> Result<(), ResponseError> {
  if !config.enabled {
    return Ok(());
  }
  if config.max_attempts < 1 {
    return Err(ResponseError::BadRequest(
      "max attempts must be greater than zero".to_owned(),
    ));
  }
  if config.fix_script.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "fix script cannot be empty".to_owned(),
    ));
  }
  if config.upload_path.trim().is_empty() || !config.upload_path.starts_with('/') {
    return Err(ResponseError::BadRequest(
      "upload path must be an absolute path".to_owned(),
    ));
  }
  if config.timeout_secs < 10 {
    return Err(ResponseError::BadRequest(
      "timeout must be at least 10 seconds".to_owned(),
    ));
  }
  if config.result_env.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "result env cannot be empty".to_owned(),
    ));
  }
  if config.success_value.trim().is_empty() {
    return Err(ResponseError::BadRequest(
      "success value cannot be empty".to_owned(),
    ));
  }
  if config.tester.is_none() {
    return Err(ResponseError::BadRequest(
      "tester image is required when fix is enabled".to_owned(),
    ));
  }
  Ok(())
}

pub(super) async fn get_fix_config(
  State(ref db): State<Database>, State(ref bucket): State<Bucket>,
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>, team_ext: Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  let challenge_bucket = super::get_challenge_bucket(bucket, &game, &challenge).await?;
  let config = challenge_bucket.fix().await?;
  let attempts_used = count_fix_attempts(&db.conn, &game, &challenge, team.as_ref(), token.id).await?;
  let solved = solved(&db.conn, &game, &challenge, team.as_ref(), token.id).await?;
  let attempts_remaining = config
    .as_ref()
    .filter(|config| config.enabled)
    .map(|config| (config.max_attempts as u64).saturating_sub(attempts_used));
  let config = if is_game_admin!(token, game) {
    config
  } else {
    config.map(FixConfig::desensitize)
  };
  Ok(Json(FixStatus {
    config,
    attempts_used,
    attempts_remaining,
    solved,
  }))
}

pub(super) async fn update_fix_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(config): Json<FixConfig>,
) -> Result<impl IntoResponse, ResponseError> {
  super::check_challenge_publishing(&challenge)?;
  validate_fix_config(&config)?;

  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  challenge_bucket
    .set_fix(serde_json::to_value(&config)?)
    .await?;
  game_bucket
    .commit(
      format!(
        ":building_construction: update fix config for challenge {}",
        challenge.name
      ),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(config))
}

pub(super) async fn delete_fix_config(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  super::check_challenge_publishing(&challenge)?;
  let (game_bucket, challenge_bucket) =
    super::get_challenge_bucket_mut(bucket, &game, &challenge).await?;
  challenge_bucket.delete_fix().await?;
  game_bucket
    .commit(
      format!(":fire: delete fix config for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn submit_fix(
  State(ref db): State<Database>, State(ref bucket): State<Bucket>, State(ref queue): State<Queue>,
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(trace): Extension<RequestId>, team_ext: Extension<Option<team::Model>>,
  Extension(challenge): Extension<challenge::Model>, mut multipart: Multipart,
) -> Result<impl IntoResponse, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  // BITs2CTF fork (B2): count attempts against the player's real team when they
  // have one (stable across game states). `store_team` is only nulled post-game
  // so the stored submission isn't attributed to the team score after the game.
  let store_team = if team.is_some()
    && game.in_progress()
    && challenge.archive_at.is_none_or(|t| t > chrono::Utc::now())
  {
    team.clone()
  } else {
    None
  };
  let challenge_bucket = super::get_challenge_bucket(bucket, &game, &challenge).await?;
  let Some(fix_config) = challenge_bucket.fix().await? else {
    return Err(ResponseError::PreconditionFailed(
      "this challenge is not a fix challenge".to_owned(),
    ));
  };
  if !fix_config.enabled {
    return Err(ResponseError::PreconditionFailed(
      "this challenge is not a fix challenge".to_owned(),
    ));
  }
  // fast fail before uploading the artifact (authoritative re-check is in the txn below).
  if solved(&db.conn, &game, &challenge, team.as_ref(), token.id).await? {
    return Err(ResponseError::Conflict(
      "challenge already solved".to_owned(),
    ));
  }
  if count_fix_attempts(&db.conn, &game, &challenge, team.as_ref(), token.id).await?
    >= fix_config.max_attempts as u64
  {
    return Err(ResponseError::PreconditionFailed(
      "fix attempts exhausted".to_owned(),
    ));
  }

  let token_id = nanoid!(21, &LABEL_ALPHABET);
  let upload_dir = fix_upload_dir(&token_id);
  tokio::fs::create_dir_all(&upload_dir).await?;
  let upload_path = fix_upload_path(&token_id);
  let mut file_name = None;
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| ResponseError::BadRequest(err.to_string()))?
  {
    if file_name.is_some() {
      warn!("fix submission contains extra files, ignoring");
      continue;
    }
    let name = field
      .file_name()
      .ok_or(ResponseError::BadRequest(
        "file name is required".to_owned(),
      ))?
      .to_owned();
    let mut reader = StreamReader::new(field.map_err(std::io::Error::other));
    let mut file = tokio::fs::File::create(&upload_path).await?;
    tokio::io::copy(&mut reader, &mut file).await?;
    file.flush().await?;
    file_name = Some(name);
  }
  let file_name = file_name.ok_or(ResponseError::BadRequest(
    "fix submission file is required".to_owned(),
  ))?;

  let content = encode_fix_submission_meta(&FixSubmissionMeta {
    token: token_id,
    file_name,
  })?;
  // BITs2CTF fork (B2): re-check + create atomically under a per-entity advisory
  // lock so concurrent uploads cannot each pass the cap and overshoot it (TOCTOU).
  let entity_key = team.as_ref().map(|team| team.id).unwrap_or(token.id) as i32;
  let txn = db.conn.begin().await?;
  txn
    .execute(Statement::from_sql_and_values(
      DatabaseBackend::Postgres,
      "SELECT pg_advisory_xact_lock($1, $2)",
      [(challenge.id as i32).into(), entity_key.into()],
    ))
    .await?;
  if solved(&txn, &game, &challenge, team.as_ref(), token.id).await? {
    txn.rollback().await.ok();
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    return Err(ResponseError::Conflict(
      "challenge already solved".to_owned(),
    ));
  }
  if count_fix_attempts(&txn, &game, &challenge, team.as_ref(), token.id).await?
    >= fix_config.max_attempts as u64
  {
    txn.rollback().await.ok();
    let _ = tokio::fs::remove_dir_all(&upload_dir).await;
    return Err(ResponseError::PreconditionFailed(
      "fix attempts exhausted".to_owned(),
    ));
  }
  let submission = submission::create(
    &txn,
    submission::Model {
      id: 0,
      created_at: chrono::Utc::now(),
      challenge_id: challenge.id,
      content: Some(content),
      solved: None,
      result: None,
      team_id: store_team.as_ref().map(|team| team.id),
      user_id: token.id,
    },
  )
  .await?;
  txn.commit().await?;
  queue
    .publish(
      "fix",
      submission.clone(),
      &trace.header_value().to_str().unwrap_or("UNKNOWN"),
    )
    .await?;
  info!(submission_id = submission.id, "submitted fix artifact");
  Ok(Json(submission))
}
