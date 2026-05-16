use axum::{
  Extension, Json,
  extract::{Multipart, State},
  response::IntoResponse,
};
use futures::TryStreamExt;
use nanoid::nanoid;
use r2s_bucket::Bucket;
use r2s_config::cluster::FixConfig;
use r2s_database::{challenge, game, submission, team, user::Permission};
use r2s_migrator::Database;
use r2s_queue::Queue;
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
  traits::ResponseError,
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

async fn count_fix_attempts(
  db: &Database, game: &game::Model, challenge: &challenge::Model, team: Option<&team::Model>,
  user_id: i64,
) -> Result<u64, ResponseError> {
  let submissions = submission::get_list(
    &db.conn,
    false,
    true,
    Some(game.id),
    Some(challenge.id),
    team.map(|team| team.id),
    if team.is_none() { Some(user_id) } else { None },
    team.is_none(),
  )
  .await?;
  Ok(
    submissions
      .into_iter()
      .filter(|submission| is_fix_submission(submission.content.as_deref()))
      .count() as u64,
  )
}

async fn solved(
  db: &Database, game: &game::Model, challenge: &challenge::Model, team: Option<&team::Model>,
  user_id: i64,
) -> Result<bool, ResponseError> {
  Ok(
    submission::count(
      &db.conn,
      true,
      Some(game.id),
      Some(challenge.id),
      team.map(|team| team.id),
      if team.is_none() { Some(user_id) } else { None },
      None,
      team.is_none(),
    )
    .await?
      > 0,
  )
}

fn validate_fix_config(config: &FixConfig) -> Result<(), ResponseError> {
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
  let attempts_used = count_fix_attempts(db, &game, &challenge, team.as_ref(), token.id).await?;
  let solved = solved(db, &game, &challenge, team.as_ref(), token.id).await?;
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
  let team = if team.is_some()
    && game.in_progress()
    && challenge.archive_at.is_none_or(|t| t > chrono::Utc::now())
  {
    team
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
  if solved(db, &game, &challenge, team.as_ref(), token.id).await? {
    return Err(ResponseError::Conflict(
      "challenge already solved".to_owned(),
    ));
  }
  let attempts_used = count_fix_attempts(db, &game, &challenge, team.as_ref(), token.id).await?;
  if attempts_used >= fix_config.max_attempts as u64 {
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
  let submission = submission::create(
    &db.conn,
    submission::Model {
      id: 0,
      created_at: chrono::Utc::now(),
      challenge_id: challenge.id,
      content: Some(content),
      solved: None,
      result: None,
      team_id: team.as_ref().map(|team| team.id),
      user_id: token.id,
    },
  )
  .await?;
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
