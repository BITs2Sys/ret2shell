use axum::{
    body::Body,
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Extension, Json, Router,
};
use chrono::Utc;
use r2s_bucket::{challenge::ChallengeBucket, Bucket};
use r2s_database::{
    challenge, extra, game, hint, team,
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
        data,
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
                .route("/files", get(get_player_attachment))
                .route(
                    "/hint",
                    post(create_challenge_hint).delete(delete_challenge_hint),
                )
                .route("/", patch(update_challenge).delete(delete_challenge))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth::game_admin_required,
                ))
                .route("/hint", get(get_challenge_hints))
                .route("/", get(get_challenge).post(submit_flag))
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
            data::prepare_team_info,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::game_access_required,
        ))
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
    if challenge.hidden
        && !(token.permissions.0.contains(&Permission::Game) && game.admins.0.contains(&token.id))
    {
        return Err(ResponseError::Forbidden(
            "permission denied".to_owned(),
            format!(
                "user {}:'{}' ({}) want to access hidden challenge {}:'{}'",
                token.id, token.account, token.nickname, challenge.id, challenge.name
            ),
        ));
    }
    if token.permissions.0.contains(&Permission::Game) && game.admins.0.contains(&token.id) {
        return Ok(Json(challenge));
    }
    Ok(Json(challenge.desensitize()))
}

async fn create_challenge(
    State(ref db): State<Database>, State(bucket): State<Bucket>,
    Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
    Json(challenge): Json<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    let txn = db.conn.begin().await?;
    let game_bucket = bucket
        .at_mut(
            game.bucket
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
    Extension(prev_challenge): Extension<challenge::Model>,
    Json(challenge): Json<challenge::Model>,
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
            game.bucket
                .ok_or(ResponseError::PreconditionFailed(format!(
                    "game {}:'{}' does not have a valid bucket",
                    game.id, game.name
                )))?,
        )
        .await?;
    let challenge_bucket = game_bucket
        .at(&challenge
            .bucket
            .clone()
            .ok_or(ResponseError::PreconditionFailed(format!(
                "challenge {}:'{}' in game {}:'{}' does not have a valid bucket",
                game.id, game.name, challenge.id, challenge.name
            )))?)
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
    State(ref db): State<Database>, State(bucket): State<Bucket>,
    Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
    Extension(challenge): Extension<challenge::Model>,
) -> Result<impl IntoResponse, ResponseError> {
    let txn = db.conn.begin().await?;
    challenge::delete(&txn, challenge.id).await?;
    let game_bucket = bucket
        .at_mut(
            game.bucket
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

async fn submit_flag() -> Result<impl IntoResponse, ResponseError> {
    Ok("not implemented")
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
    State(ref db): State<Database>, State(ref bucket): State<Bucket>,
    Extension(game): Extension<game::Model>, Extension(challenge): Extension<challenge::Model>,
    Extension(token): Extension<Token>, Query(query): Query<FileRequest>,
) -> Result<Response, ResponseError> {
    let challenge_bucket = bucket
        .at(&game
            .bucket
            .clone()
            .ok_or(ResponseError::PreconditionFailed(format!(
                "game {}:'{}' does not have a valid bucket",
                game.id, game.name
            )))?)
        .await?
        .at(&challenge
            .bucket
            .clone()
            .ok_or(ResponseError::PreconditionFailed(format!(
                "game {}:'{}' does not have a valid bucket",
                game.id, game.name
            )))?)
        .await?;
    let files = get_files(db, &challenge_bucket, game, token).await?;
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

        let stream = ReaderStream::new(file);
        Ok((StatusCode::OK, Body::from_stream(stream)).into_response())
    }
}

async fn get_files(
    db: &Database, bucket: &ChallengeBucket, game: game::Model, token: Token,
) -> Result<Vec<FileResponse>, ResponseError> {
    let static_files = bucket.get_static_files().await?;
    debug!("files: {:?}", static_files);
    let mapped_file = if game.in_progress() {
        let team = team::get_by_user_id(&db.conn, game.id, token.id).await?;
        if team.is_none()
            || team
                .clone()
                .is_some_and(|team| team.state == team::State::Banned)
        {
            return Err(ResponseError::Forbidden(
                "permission denied".to_owned(),
                format!(
                    "user {}:'{}' ({}) want to access game {}:'{}' api with out participation or banned",
                    token.id, token.account, token.nickname, game.id, game.name
                ),
            ));
        }
        let team = team.unwrap();
        bucket.get_mapped_file(team.id).await?
    } else {
        bucket.get_mapped_file(token.id).await?
    };
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
    if game.start_at < Utc::now() {
        return Err(ResponseError::Forbidden(
            "game is not start".to_owned(),
            format!("user {}:'{}' ({}) wants to access challenge {}:'{}' hint before game {}:'{}' start", token.id, token.account, token.nickname, challenge.id, challenge.name, game.id, game.name))
        );
    }
    let team = if game.in_progress()
        && !(game.admins.0.contains(&token.id) && token.permissions.0.contains(&Permission::Game))
    {
        if let Some(Extension(team)) = team_ext {
            Some(team)
        } else {
            return Err(ResponseError::Forbidden(
                "please take part in first".to_owned(),
                format!(
                    "user {}:'{}' ({}) wants to access challenge {}:'{}' hint without permission",
                    token.id, token.account, token.nickname, challenge.id, challenge.name
                ),
            ));
        }
    } else {
        None
    };
    let hints = hint::get_list(&db.conn, challenge.id).await?;
    if let Some(team) = team {
        let extras = extra::get_list(&db.conn, team.id).await?;
        let hints = hints
            .iter()
            .map(|h| {
                if h.cost > 0 && extras.iter().find(|e| e.hint_id == Some(h.id)).is_none() {
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
