use axum::{
  body::Body,
  extract::{DefaultBodyLimit, Multipart, Query, State},
  http::{HeaderMap, StatusCode},
  middleware,
  response::{IntoResponse, Response},
  routing::{get, patch, post},
  Extension, Json, Router,
};
use chrono::Utc;
use futures::TryStreamExt;
use nanoid::nanoid;
use r2s_bucket::{challenge::ChallengeBucket, Bucket};
use r2s_cache::Cache;
use r2s_checker::{traits::CheckerError, Checker};
use r2s_cluster::{Cluster, CHALLENGE_NS};
use r2s_config::{cluster::ChallengeEnv, GlobalConfig};
use r2s_database::{
  challenge, extra, game, hint, submission, team,
  user::{self, Permission},
};
use r2s_event::{
  events::{ChallengeEvent, ChallengeEventType, EventContainer},
  Event,
};
use r2s_migrator::Database;
use r2s_queue::Queue;
use sea_orm::{DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::{debug, warn};

use crate::{
  middleware::{
    auth::{self, is_game_admin, Token},
    data::{self, extract_team},
  },
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/", post(create_challenge))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      auth::game_admin_required,
    ))
    .route("/", get(get_challenge_list))
    .nest(
      "/:challenge",
      Router::new()
        .nest(
          "/file",
          Router::new()
            .route(
              "/",
              post(upload_challenge_attachment).delete(delete_challenge_attachment),
            )
            .route_layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
        )
        .route(
          "/env",
          patch(update_challenge_env).delete(delete_challenge_env),
        )
        .route("/instance", get(get_all_running_instances_for_challenge))
        .route(
          "/checker",
          get(get_checker_script).patch(update_checker_script),
        )
        .route(
          "/hint",
          post(create_challenge_hint).delete(delete_challenge_hint),
        )
        .route("/", patch(update_challenge).delete(delete_challenge))
        .route("/publish", post(up_challenge).delete(down_challenge))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          auth::game_admin_required,
        ))
        .route("/file", get(get_player_attachment))
        .route("/env", get(get_challenge_env).post(start_challenge_env))
        .route("/hint", get(get_challenge_hints))
        .route("/hint/unlock", post(unlock_hint))
        .route(
          "/submit",
          get(get_challenge_solves_status).post(submit_flag),
        )
        .route("/", get(get_challenge))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          auth::challenge_access_required,
        ))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_data!(challenge, true),
        )),
    )
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      auth::game_access_required,
    ))
}

macro_rules! get_challenge_bucket {
  ($bucket:expr, $game:expr, $challenge:expr) => {{
    $bucket
      .at(
        $game
          .bucket
          .ok_or(ResponseError::PreconditionFailed(format!(
            "game {}:'{}' does not have a valid bucket",
            $game.id, $game.name
          )))?,
      )
      .await?
      .at(
        $challenge
          .bucket
          .ok_or(ResponseError::PreconditionFailed(format!(
            "challenge {}:'{}' in game {}:'{}' does not have a valid bucket",
            $challenge.id, $challenge.name, $game.id, $game.name
          )))?,
      )
      .await?
  }};
}

macro_rules! get_challenge_bucket_mut {
  ($bucket:expr, $game:expr, $challenge:expr) => {{
    if !$challenge.hidden {
      return Err(ResponseError::PreconditionFailed(
        "please hidden challenge before update it".to_owned(),
      ));
    }
    let game_bucket = $bucket
      .at_mut(
        $game
          .bucket
          .ok_or(ResponseError::PreconditionFailed(format!(
            "game {}:'{}' does not have a valid bucket",
            $game.id, $game.name
          )))?,
      )
      .await?;
    let challenge_bucket = game_bucket
      .at(
        $challenge
          .bucket
          .clone()
          .ok_or(ResponseError::PreconditionFailed(format!(
            "challenge {}:'{}' in game {}:'{}' does not have a valid bucket",
            $challenge.id, $challenge.name, $game.id, $game.name
          )))?,
      )
      .await?;
    (game_bucket, challenge_bucket)
  }};
}

// macro_rules! check_const_columns {
//   ($prev_model:expr, $current_model:expr, $($columns:tt), *) => {{
//     $(
//       if $prev_model.$columns != $current_model.$columns {
//         return Err(ResponseError::PreconditionFailed(format!(
//           "column {} is not allowed to change",
//           stringify!($columns)
//         )));
//       }
//     )*
//   }};
// }

async fn update_team_state(db: &Database, team: team::Model) {
  let score = team::calc_score(&db.conn, team.id).await;
  if score.is_err() {
    warn!("calc team score failed: {:?}", score.err());
    return;
  }
  let score = score.unwrap();
  if score != team.score {
    let mut team_history = team.history.clone().0;
    team_history.push(team::TeamScoreHistory {
      score,
      challenge_id: None,
      changed_at: Utc::now(),
      blood_state: None,
    });
    let result = team::update(
      &db.conn,
      team::Model {
        id: team.id,
        score,
        history: team::TeamScoreHistoryList(team_history),
        ..team
      },
    )
    .await;
    if let Err(e) = result {
      warn!("update team score failed: {:?}", e);
    }
  }
}

#[derive(Deserialize)]
struct ChallengeQuery {
  page: Option<u64>,
  page_size: Option<u64>,
}

async fn get_challenge_list(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Query(query): Query<ChallengeQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let with_hidden = is_game_admin!(token, game);
  if query.page.is_none() || query.page_size.is_none() {
    let challenges = challenge::get_list(&db.conn, game.id, with_hidden).await?;
    return Ok(Json((challenges, 1)));
  }
  let page = query.page.unwrap_or(1);
  let page_size = query.page_size.unwrap_or(15);
  Ok(Json(
    challenge::get_page(&db.conn, page, page_size, game.id, with_hidden).await?,
  ))
}

async fn get_challenge(
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if is_game_admin!(token, game) {
    return Ok(Json(challenge));
  }
  if challenge.hidden {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:'{}' ({}) want to access hidden challenge {}:'{}'",
        token.id, token.account, token.nickname, challenge.id, challenge.name
      ),
    ));
  }

  Ok(Json(challenge.desensitize()))
}

async fn create_challenge(
  State(ref db): State<Database>, State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Json(challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let game_bucket = bucket
    .at_mut(
      game
        .bucket
        .ok_or(ResponseError::PreconditionFailed(format!(
          "game {}:'{}' does not have a valid bucket",
          game.id, game.name
        )))?,
    )
    .await?;
  let challenge_bucket = game_bucket
    .create(serde_json::to_value(&challenge)?)
    .await?;
  challenge_bucket
    .set_description(challenge.content.clone().unwrap_or_default())
    .await?;
  let challenge = challenge::create(
    &txn,
    challenge::Model {
      game_id: game.id,
      hidden: true,
      bucket: Some(challenge_bucket.name),
      ..challenge
    },
  )
  .await?;
  game_bucket
    .commit(
      format!("create challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  txn.commit().await?;

  Ok(Json(challenge))
}

async fn update_challenge(
  State(ref db): State<Database>, State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(prev_challenge): Extension<challenge::Model>,
  Json(challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  // refuse to change some columns when challenge is cloned from another challenge.
  // if prev_challenge.ref_id.is_some() {
  //   check_const_columns!(
  //     prev_challenge,
  //     challenge,
  //     bucket,
  //     ref_id,
  //     name,
  //     tag,
  //     score_rule,
  //     content
  //   );
  // }
  let txn = db.conn.begin().await?;
  let challenge = challenge::update(
    &txn,
    challenge::Model {
      hidden: prev_challenge.hidden,
      ..challenge
    },
  )
  .await?;
  // if challenge.ref_id.is_none() {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  challenge_bucket
    .set_config(serde_json::to_value(&challenge)?)
    .await?;
  challenge_bucket
    .set_description(challenge.content.clone().unwrap_or_default())
    .await?;

  game_bucket
    .commit(
      format!("update challenge config {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  // }
  txn.commit().await?;

  Ok(Json(challenge))
}

async fn up_challenge(
  State(ref db): State<Database>, State(bucket): State<Bucket>, State(ref queue): State<Queue>,
  State(checker): State<Checker>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let challenge_bucket = get_challenge_bucket!(bucket, game, challenge.clone());
  let challenge = challenge::update(
    &txn,
    challenge::Model {
      hidden: false,
      ..challenge
    },
  )
  .await?;
  match checker.lint(&challenge_bucket).await {
    Ok(_) => {}
    Err(e) => {
      return Err(ResponseError::PreconditionFailed(e.to_string()));
    }
  }
  txn.commit().await?;
  let event = EventContainer {
    game_id: challenge.game_id,
    event: Event::Challenge(ChallengeEvent {
      event_type: ChallengeEventType::Up,
      challenge: challenge.clone(),
      operator: user::Model {
        id: token.id,
        nickname: token.nickname.clone(),
        account: token.account.clone(),
        ..Default::default()
      },
    }),
  };
  queue.publish("event", event).await.ok();
  Ok(Json(challenge))
}

async fn down_challenge(
  State(ref db): State<Database>, State(ref queue): State<Queue>,
  Extension(token): Extension<Token>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let challenge = challenge::update(
    &txn,
    challenge::Model {
      hidden: true,
      ..challenge.clone()
    },
  )
  .await?;
  txn.commit().await?;
  let event = EventContainer {
    game_id: challenge.game_id,
    event: Event::Challenge(ChallengeEvent {
      event_type: ChallengeEventType::Down,
      challenge: challenge.clone(),
      operator: user::Model {
        id: token.id,
        nickname: token.nickname.clone(),
        account: token.account.clone(),
        ..Default::default()
      },
    }),
  };
  queue.publish("event", event).await.ok();
  Ok(Json(challenge))
}

async fn delete_challenge(
  State(ref db): State<Database>, State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  challenge::delete(&txn, challenge.id).await?;
  // if challenge.ref_id.is_none() {
  let game_bucket = bucket
    .at_mut(
      game
        .bucket
        .ok_or(ResponseError::PreconditionFailed(format!(
          "game {}:'{}' does not have a valid bucket",
          game.id, game.name
        )))?,
    )
    .await?;
  game_bucket
    .delete(
      &challenge
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "challenge {}:'{}' in game {}:'{}' does not have a valid bucket",
          game.id, game.name, challenge.id, challenge.name
        )))?,
    )
    .await?;
  game_bucket
    .commit(
      format!("delete challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  // }
  txn.commit().await?;

  Ok(())
}

#[derive(Serialize)]
struct SolvesStatus {
  pub solved: bool,
  pub solves: u64,
  pub top: Vec<submission::ExModel>,
}

async fn get_challenge_solves_status(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, team_ext: Option<Extension<team::Model>>,
  Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  let (top, solves) =
    submission::get_page_ex(&db.conn, 1, 3, true, false, Some(challenge.id), None, None).await?;
  let solved = if let Some(team) = team {
    submission::count(&db.conn, true, Some(challenge.id), Some(team.id), None).await? > 0
  } else {
    submission::count(&db.conn, true, Some(challenge.id), None, Some(token.id)).await? > 0
  };
  Ok(Json(SolvesStatus {
    solved,
    solves,
    top,
  }))
}

#[derive(Deserialize)]
struct SubmitRequest {
  pub content: String,
}

#[allow(clippy::too_many_arguments)]
async fn submit_flag(
  State(ref db): State<Database>, State(_bucket): State<Bucket>, State(ref queue): State<Queue>,
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  team_ext: Option<Extension<team::Model>>, Extension(challenge): Extension<challenge::Model>,
  Json(req): Json<SubmitRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  // Only game in progress and has a team, the submission record will be marked as
  // team's. otherwise the submission will only be marked as user-solved.
  let team = extract_team!(game, team_ext, token);
  let submission = submission::Model {
    id: 0,
    created_at: Utc::now(),
    challenge_id: challenge.id,
    content: Some(req.content),
    solved: None,
    result: None,
    team_id: if let Some(team) = team {
      Some(team.id)
    } else {
      None
    },
    user_id: token.id,
  };
  let submission = submission::create(&db.conn, submission).await?;
  queue.publish("check", submission).await?;
  Ok(())
}

// TODO: publish solved event
// TODO: update scoreboard
// TODO: check flag

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
enum FileType {
  Static,
  Mapped,
  Checker,
}

#[derive(Deserialize)]
struct FileRequest {
  pub folder: Option<FileType>,
  pub file: Option<String>,
  pub all: Option<bool>,
}

#[derive(Serialize)]
struct FileResponse {
  pub folder: FileType,
  pub file: String,
}

async fn get_player_attachment(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>, Extension(token): Extension<Token>,
  team_ext: Option<Extension<team::Model>>, Query(query): Query<FileRequest>,
) -> Result<Response, ResponseError> {
  let challenge_bucket = get_challenge_bucket!(bucket, game.clone(), challenge);
  if !is_game_admin!(token, game)
    && (query.all == Some(true) || query.folder == Some(FileType::Checker))
  {
    return Err(ResponseError::Forbidden(
      "permission denied".to_owned(),
      format!(
        "user {}:'{}' ({}) want to access checker files",
        token.id, token.account, token.nickname
      ),
    ));
  }
  if query.all == Some(true) && is_game_admin!(token, game) {
    let files = match query.folder {
      Some(FileType::Static) => challenge_bucket.get_static_files().await?,
      Some(FileType::Mapped) => challenge_bucket.get_mapped_files().await?,
      Some(FileType::Checker) => challenge_bucket.get_checker_files().await?,
      None => {
        return Err(ResponseError::BadRequest("folder is required".to_owned()));
      }
    };
    let files: Vec<FileResponse> = files
      .into_iter()
      .map(|file| FileResponse {
        folder: query.folder.clone().unwrap(),
        file,
      })
      .collect();
    return Ok(Json(files).into_response());
  }
  let team = extract_team!(game, team_ext, token);
  let files = get_files(
    &challenge_bucket,
    if let Some(team) = team {
      team.id
    } else {
      token.id
    },
  )
  .await?;
  if query.file.is_none() || query.folder.is_none() {
    Ok(Json(files).into_response())
  } else {
    let file = query.file.unwrap();
    let folder = query.folder.unwrap();
    let checked_file = files
      .into_iter()
      .find(|f| f.folder == folder && f.file == file);
    if checked_file.is_none() {
      return Err(ResponseError::NotFound("file".to_string()));
    }
    let file = match folder {
      FileType::Static => challenge_bucket.download_static(&file).await?,
      FileType::Mapped => challenge_bucket.download_mapped(&file).await?,
      FileType::Checker => challenge_bucket.download_checker(&file).await?,
    };

    let mut header = HeaderMap::new();
    header.insert("Content-Length", file.metadata().await?.len().into());

    let stream = ReaderStream::new(file);
    Ok((StatusCode::OK, header, Body::from_stream(stream)).into_response())
  }
}

async fn get_files(bucket: &ChallengeBucket, id: i64) -> Result<Vec<FileResponse>, ResponseError> {
  let static_files = bucket.get_static_files().await?;
  debug!("files: {:?}", static_files);

  let mapped_file = bucket.get_mapped_file(id).await?;
  let mut files: Vec<FileResponse> = static_files
    .into_iter()
    .map(|file| FileResponse {
      folder: FileType::Static,
      file,
    })
    .collect();
  if let Some(mapped_file) = mapped_file {
    files.push(FileResponse {
      folder: FileType::Mapped,
      file: mapped_file,
    });
  }
  Ok(files)
}

#[derive(Deserialize)]
struct UploadChallengeAttachmentQuery {
  pub folder: FileType,
}

async fn upload_challenge_attachment(
  State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Query(query): Query<UploadChallengeAttachmentQuery>, mut multipart: Multipart,
) -> Result<impl IntoResponse, ResponseError> {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|err| ResponseError::BadRequest(err.to_string()))?
  {
    let file_name = field
      .file_name()
      .ok_or(ResponseError::BadRequest(
        "file name is required".to_owned(),
      ))?
      .to_owned();
    let reader =
      StreamReader::new(field.map_err(|multipart_error| {
        std::io::Error::new(std::io::ErrorKind::Other, multipart_error)
      }));
    match query.folder {
      FileType::Static => challenge_bucket.upload_static(&file_name, reader).await?,
      FileType::Mapped => challenge_bucket.upload_mapped(&file_name, reader).await?,
      FileType::Checker => challenge_bucket.upload_checker(&file_name, reader).await?,
    }
  }
  game_bucket
    .commit(
      format!("upload files for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

async fn delete_challenge_attachment(
  State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Query(query): Query<FileRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  let file = query
    .file
    .clone()
    .ok_or(ResponseError::BadRequest("file is required".to_owned()))?;
  match query.folder {
    Some(FileType::Static) => challenge_bucket.delete_static(&file).await?,
    Some(FileType::Mapped) => challenge_bucket.delete_mapped(&file).await?,
    Some(FileType::Checker) => challenge_bucket.delete_checker(&file).await?,
    None => {
      return Err(ResponseError::BadRequest("folder is required".to_owned()));
    }
  };
  game_bucket
    .commit(
      format!("delete file {} for challenge {}", file, challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

async fn get_challenge_hints(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  team_ext: Option<Extension<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = extract_team!(game, team_ext, token);
  let hints = hint::get_list(&db.conn, challenge.id).await?;
  if let Some(team) = team {
    let extras = extra::get_list(&db.conn, team.id).await?;
    let hints = hints
      .iter()
      .map(|h| {
        if h.cost > 0 && !extras.iter().any(|e| e.hint_id == Some(h.id)) {
          h.clone().desensitize()
        } else {
          h.clone()
        }
      })
      .collect();
    Ok(Json(hints))
  } else {
    Ok(Json(hints))
  }
}

#[derive(Deserialize)]
struct UnlockHintRequest {
  pub id: i64,
}

async fn unlock_hint(
  State(db): State<Database>, Extension(team): Extension<team::Model>,
  Extension(challenge): Extension<challenge::Model>, Json(req): Json<UnlockHintRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  let hint = hint::get(&txn, req.id).await?;
  if let Some(hint) = hint {
    if hint.challenge_id != challenge.id {
      return Err(ResponseError::PreconditionFailed(
        "hint does not belong to this challenge".to_owned(),
      ));
    }
    if hint.cost > team.score {
      return Err(ResponseError::PreconditionFailed(
        "you does not have enough score to unlock this hint".to_owned(),
      ));
    }
    let extra = extra::create(
      &txn,
      extra::Model {
        created_at: Utc::now(),
        score: -hint.cost,
        team_id: team.id,
        challenge_id: Some(challenge.id),
        hint_id: Some(hint.id),
        reason: format!(
          "unlocked hint [{}] in challenge {}",
          hint.id, challenge.name
        ),
        ..Default::default()
      },
    )
    .await?;
    txn.commit().await?;
    tokio::spawn(async move {
      update_team_state(&db, team).await;
    });
    Ok(Json(extra))
  } else {
    Err(ResponseError::NotFound("hint".to_string()))
  }
}

async fn create_challenge_hint(
  State(bucket): State<Bucket>, State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(hint): Json<hint::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  // if challenge.ref_id.is_some() {
  //   return Err(ResponseError::PreconditionFailed(
  //     "cannot create hint for cloned challenge".to_owned(),
  //   ));
  // }
  let txn = db.conn.begin().await?;
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);

  let hint = hint::create(
    &txn,
    hint::Model {
      challenge_id: challenge.id,
      ..hint
    },
  )
  .await?;
  sync_challenge_hint_with_bucket(&challenge_bucket, &txn, &challenge).await?;
  txn.commit().await?;
  game_bucket
    .commit(
      format!("new hint for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(Json(hint))
}

#[derive(Deserialize)]
struct DeleteHintQuery {
  pub id: i64,
}

async fn delete_challenge_hint(
  State(bucket): State<Bucket>, State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Query(query): Query<DeleteHintQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  // if challenge.ref_id.is_some() {
  //   return Err(ResponseError::PreconditionFailed(
  //     "cannot delete hint for cloned challenge".to_owned(),
  //   ));
  // }
  let txn = db.conn.begin().await?;
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  hint::delete(&txn, query.id).await?;
  sync_challenge_hint_with_bucket(&challenge_bucket, &txn, &challenge).await?;
  txn.commit().await?;
  game_bucket
    .commit(
      format!("delete hint for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

async fn sync_challenge_hint_with_bucket(
  bucket: &ChallengeBucket, db: &DatabaseTransaction, challenge: &challenge::Model,
) -> Result<(), ResponseError> {
  let hints = hint::get_list(db, challenge.id).await?;
  bucket
    .set_hints(
      hints
        .into_iter()
        .map(|m| r2s_bucket::Hint {
          content: m.content,
          cost: m.cost,
        })
        .collect(),
    )
    .await?;
  Ok(())
}

async fn get_challenge_env(
  State(ref bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = get_challenge_bucket!(bucket, game, challenge);
  let env_config = challenge_bucket.env().await?;
  if is_game_admin!(token, game) {
    Ok(Json(env_config))
  } else {
    Ok(Json(env_config.map(|c| c.desensitize())))
  }
}

#[allow(clippy::too_many_arguments)]
#[axum::debug_handler(state = GlobalState)]
async fn start_challenge_env(
  State(config): State<GlobalConfig>, State(bucket): State<Bucket>, State(cluster): State<Cluster>,
  State(cache): State<Cache>, State(mut checker): State<Checker>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Extension(token): Extension<Token>, team_ext: Option<Extension<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let config = if let Some(config) = &config.cluster {
    config
  } else {
    return Err(ResponseError::PreconditionFailed(
      "cluster is disabled".to_owned(),
    ));
  };
  let calmdown = cache.at("cluster").exists(token.id.to_string()).await?;
  if calmdown {
    return Err(ResponseError::PreconditionFailed(
      "please wait for rebuilding cargo crates".to_owned(),
    ));
  }
  let challenge_bucket = get_challenge_bucket!(bucket, game.clone(), challenge.clone());
  let team = extract_team!(game, team_ext, token);
  if let Some(env_config) = challenge_bucket.env().await? {
    checker.preload(&challenge, &challenge_bucket).await?;
    let env_map = checker
      .environ(
        &challenge_bucket,
        &user::Model {
          id: token.id.clone(),
          nickname: token.nickname.clone(),
          account: token.account.clone(),
          ..Default::default()
        },
        &team,
      )
      .await?;
    cluster
      .at(CHALLENGE_NS)
      .create_challenge_env(
        [
          ("ret.sh.cn/challenge", challenge.id.to_string()),
          (
            "ret.sh.cn/team",
            team.clone().map(|t| t.id.to_string()).unwrap_or_default(),
          ),
          ("ret.sh.cn/game", game.id.to_string()),
          ("ret.sh.cn/user", token.id.to_string()),
          ("ret.sh.cn/wsrx", nanoid!()),
          ("ret.sh.cn/renew", 0.to_string()),
          ("ret.sh.cn/internet", env_config.internet.to_string()),
        ]
        .iter()
        .cloned()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect(),
        [
          ("ret.sh.cn/challenge", challenge.name.to_string()),
          (
            "ret.sh.cn/team",
            team.map(|t| t.name.to_string()).unwrap_or_default(),
          ),
          ("ret.sh.cn/game", game.name.to_string()),
          ("ret.sh.cn/user", token.account.to_string()),
        ]
        .iter()
        .cloned()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect(),
        env_map.into(),
        env_config,
        config.challenge_node_selector.clone(),
      )
      .await?;
    cache
      .at("cluster")
      .set_ex(token.id.to_string(), 1, 60)
      .await?;
    Ok(())
  } else {
    Err(ResponseError::PreconditionFailed(
      "challenge does not have online environment".to_owned(),
    ))
  }
}

async fn update_challenge_env(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(token): Extension<Token>, Extension(challenge): Extension<challenge::Model>,
  Json(env): Json<ChallengeEnv>,
) -> Result<impl IntoResponse, ResponseError> {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  challenge_bucket
    .set_env(serde_json::to_value(&env)?)
    .await?;
  game_bucket
    .commit(
      format!("update env for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;

  Ok(())
}

async fn delete_challenge_env(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(token): Extension<Token>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  challenge_bucket.delete_env().await?;
  game_bucket
    .commit(
      format!("delete env for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

#[derive(Serialize)]
struct CheckerResponse {
  pub script: String,
  pub lint: Option<String>,
}

#[derive(Deserialize)]
struct CheckerRequest {
  pub lint: Option<bool>,
}

async fn get_checker_script(
  State(ref bucket): State<Bucket>, State(checker): State<Checker>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Query(query): Query<CheckerRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = get_challenge_bucket!(bucket, game, challenge);
  let lint = if let Some(true) = query.lint {
    let lint = checker.lint(&challenge_bucket).await;
    if let Err(lint) = lint {
      match lint {
        CheckerError::CompileError(diagnostics) => Some(diagnostics),
        err => {
          warn!("failed to lint script: {:?}", err);
          Some(err.to_string())
        }
      }
    } else {
      None
    }
  } else {
    None
  };

  Ok(Json(CheckerResponse {
    script: challenge_bucket.checker().await?,
    lint,
  }))
}

#[derive(Deserialize)]
struct UpdateCheckerScriptRequest {
  pub content: String,
}

async fn update_checker_script(
  State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
  Json(req): Json<UpdateCheckerScriptRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let (game_bucket, challenge_bucket) = get_challenge_bucket_mut!(bucket, game, challenge);
  challenge_bucket.set_checker(req.content).await?;
  game_bucket
    .commit(
      format!("update checker script for challenge {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  Ok(())
}

async fn get_all_running_instances_for_challenge(
  State(ref cluster): State<Cluster>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let instances = cluster
    .at(CHALLENGE_NS)
    .get_challenge_env(challenge.id)
    .await?;
  Ok(Json(instances))
}
