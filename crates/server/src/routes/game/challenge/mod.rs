use axum::{
  body::Body,
  extract::{Query, State},
  http::{HeaderMap, StatusCode},
  middleware,
  response::{IntoResponse, Response},
  routing::{get, patch, post},
  Extension, Json, Router,
};
use chrono::Utc;
use r2s_bucket::{challenge::ChallengeBucket, Bucket};
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
use sea_orm::TransactionTrait;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tracing::debug;

use crate::{
  middleware::{
    auth::{self, Token},
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
        .route(
          "/hint",
          post(create_challenge_hint).delete(delete_challenge_hint),
        )
        .route("/", patch(update_challenge).delete(delete_challenge))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          auth::game_admin_required,
        ))
        .route("/files", get(get_player_attachment))
        .route("/env", get(get_challenge_env))
        .route("/hint", get(get_challenge_hints))
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

#[derive(Deserialize)]
struct ChallengeQuery {
  page: Option<u64>,
  page_size: Option<u64>,
}

async fn get_challenge_list(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Query(query): Query<ChallengeQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let with_hidden =
    game.admins.0.contains(&token.id) && token.permissions.0.contains(&Permission::Game);
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
  if token.permissions.0.contains(&Permission::Game) && game.admins.0.contains(&token.id) {
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
  State(ref db): State<Database>, State(bucket): State<Bucket>, State(ref queue): State<Queue>,
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(prev_challenge): Extension<challenge::Model>, Json(challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  if !prev_challenge.hidden && !challenge.hidden {
    return Err(ResponseError::PreconditionFailed(
      "please hidden challenge before update it".to_owned(),
    ));
  }
  let txn = db.conn.begin().await?;
  let challenge = challenge::update(
    &txn,
    challenge::Model {
      id: prev_challenge.id,
      game_id: prev_challenge.game_id,
      bucket: prev_challenge.bucket,
      ..challenge
    },
  )
  .await?;
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
    .at(
      &challenge
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "challenge {}:'{}' in game {}:'{}' does not have a valid bucket",
          game.id, game.name, challenge.id, challenge.name
        )))?,
    )
    .await?;
  challenge_bucket
    .set_config(serde_json::to_value(&challenge)?)
    .await?;

  game_bucket
    .commit(
      format!("update challenge config {}", challenge.name),
      &token.account,
      format!("{}@private.ret.sh.cn", token.account),
    )
    .await?;
  txn.commit().await?;
  if prev_challenge.hidden != challenge.hidden {
    let event = EventContainer {
      game_id: game.id,
      event: Event::Challenge(ChallengeEvent {
        event_type: if prev_challenge.hidden {
          ChallengeEventType::Up
        } else {
          ChallengeEventType::Down
        },
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
  }

  Ok(Json(challenge))
}

async fn delete_challenge(
  State(ref db): State<Database>, State(bucket): State<Bucket>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let txn = db.conn.begin().await?;
  challenge::delete(&txn, challenge.id).await?;
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
  State(ref db): State<Database>, State(_bucket): State<Bucket>, State(ref _queue): State<Queue>,
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
  submission::create(&db.conn, submission).await?;
  // TODO: publish solved event
  // TODO: update scoreboard
  // TODO: check flag
  Ok("TODO: check flag")
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FileType {
  Static,
  Mapped,
}

#[derive(Deserialize)]
struct FileRequest {
  pub folder: Option<FileType>,
  pub file: Option<String>,
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
  let challenge_bucket = bucket
    .at(
      &game
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "game {}:'{}' does not have a valid bucket",
          game.id, game.name
        )))?,
    )
    .await?
    .at(
      &challenge
        .bucket
        .clone()
        .ok_or(ResponseError::PreconditionFailed(format!(
          "game {}:'{}' does not have a valid bucket",
          game.id, game.name
        )))?,
    )
    .await?;
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

async fn create_challenge_hint(
  State(ref db): State<Database>, Extension(challenge): Extension<challenge::Model>,
  Json(hint): Json<hint::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(
    hint::create(
      &db.conn,
      hint::Model {
        challenge_id: challenge.id,
        ..hint
      },
    )
    .await?,
  ))
}

#[derive(Deserialize)]
struct DeleteHintQuery {
  pub id: i64,
}

async fn delete_challenge_hint(
  State(ref db): State<Database>, Query(query): Query<DeleteHintQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  hint::delete(&db.conn, query.id).await?;
  Ok(())
}

async fn get_challenge_env(
  State(ref bucket): State<Bucket>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let challenge_bucket = get_challenge_bucket!(bucket, game, challenge);
  let env_config = challenge_bucket.env().await?;

  Ok(Json(env_config))
}
