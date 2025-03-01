use axum::{
  Extension, Json, Router,
  extract::{Query, State},
  middleware,
  response::IntoResponse,
  routing::get,
};
use r2s_database::{challenge, chat, game, team, user};
use r2s_event::{
  Event,
  events::{ChatEvent, ChatEventType, EventContainer},
};
use r2s_migrator::Database;
use r2s_queue::Queue;
use serde::Deserialize;

use crate::{
  middleware::{
    auth::{self, Token},
    data,
  },
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .nest(
      "/admin",
      Router::new()
        .route("/", get(admin_get_chat_list))
        .route(
          "/session",
          get(admin_get_chat_session).post(admin_send_chat),
        )
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          auth::game_admin_required,
        )),
    )
    .nest(
      "/{challenge}",
      Router::new()
        .route("/", get(player_get_chat_session).post(player_send_chat))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_data!(challenge, true),
        )),
    )
    .route("/unread", get(check_unread_chats))
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      auth::game_access_required,
    ))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      user::Permission::Basic,
      user::Permission::Verified
    )))
}

#[derive(Deserialize)]
struct SendChatRequest {
  content: String,
}

#[derive(Deserialize)]
struct GetChatListQuery {
  page: Option<u64>,
  page_size: Option<u64>,
}

async fn admin_get_chat_list(
  State(ref db): State<Database>, Extension(game): Extension<game::Model>,
  Query(query): Query<GetChatListQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let chats = chat::get_sessions(
    &db.conn,
    game.id,
    query.page.unwrap_or(1),
    query.page_size.unwrap_or(30),
  )
  .await?;
  Ok(Json(chats))
}

async fn player_get_chat_session(
  State(ref db): State<Database>, Extension(challenge): Extension<challenge::Model>,
  Extension(token): Extension<Token>, Extension(team): Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = team.ok_or_else(|| {
    ResponseError::Forbidden(
      "team not found".into(),
      format!(
        "user {}:'{}' ({}) want to access chat session without participate game",
        token.id, token.account, token.nickname
      ),
    )
  })?;
  let chats = chat::get_list(&db.conn, team.id, challenge.id).await?;
  if chats.first().is_some_and(|c| c.is_admin && !c.checked) {
    chat::mark_checked(&db.conn, team.id, challenge.id).await?;
  }
  Ok(Json(chats))
}

async fn player_send_chat(
  State(ref db): State<Database>, State(ref queue): State<Queue>,
  Extension(token): Extension<Token>, Extension(game): Extension<game::Model>,
  Extension(challenge): Extension<challenge::Model>,
  Extension(team): Extension<Option<team::Model>>, Json(chat): Json<SendChatRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = team.ok_or_else(|| ResponseError::NotFound("team not found".to_owned()))?;
  let chats = chat::get_list(&db.conn, team.id, challenge.id).await?;
  let mut sent_count = 3;
  for i in chats {
    if i.is_admin {
      break;
    }
    if i.user_id == token.id {
      sent_count -= 1;
    }
    if sent_count == 0 {
      break;
    }
  }
  if sent_count <= 0 {
    return Err(ResponseError::TooManyRequests(
      "please wait for administrator's reply".into(),
      format!(
        "user {}:'{}' ({}) try to send multiple chats to challenge {}:{} in game {}:{}",
        token.id, token.account, token.nickname, challenge.id, challenge.name, game.id, game.name
      ),
    ));
  }
  chat::create(
    &db.conn,
    chat::Model {
      team_id: team.id,
      challenge_id: challenge.id,
      user_id: token.id,
      content: chat.content.clone(),
      game_id: challenge.game_id,
      checked: false,
      ..Default::default()
    },
  )
  .await?;
  let event = EventContainer {
    game_id: game.id,
    event: Event::Chat(Box::new(ChatEvent {
      operator: user::Model {
        id: token.id,
        account: token.account.clone(),
        nickname: token.nickname.clone(),
        ..Default::default()
      },
      team,
      challenge,
      event_type: ChatEventType::Message,
      content: chat.content,
    })),
  };
  queue.publish("event", event).await.ok();

  Ok(())
}

#[derive(Deserialize)]
struct AdminSessionQuery {
  team_id: i64,
  challenge_id: i64,
}

async fn admin_get_chat_session(
  State(ref db): State<Database>, Query(query): Query<AdminSessionQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let chats = chat::get_list(&db.conn, query.team_id, query.challenge_id).await?;
  if chats.first().is_some_and(|c| !c.is_admin && !c.checked) {
    chat::mark_checked(&db.conn, query.team_id, query.challenge_id).await?;
  }
  Ok(Json(chats))
}

async fn admin_send_chat(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(game): Extension<game::Model>, Query(query): Query<AdminSessionQuery>,
  Json(chat): Json<SendChatRequest>,
) -> Result<impl IntoResponse, ResponseError> {
  chat::create(
    &db.conn,
    chat::Model {
      team_id: query.team_id,
      challenge_id: query.challenge_id,
      user_id: token.id,
      content: chat.content,
      game_id: game.id,
      checked: false,
      is_admin: true,
      ..Default::default()
    },
  )
  .await?;
  Ok(())
}

async fn check_unread_chats(
  State(ref db): State<Database>, Extension(team): Extension<Option<team::Model>>,
) -> Result<impl IntoResponse, ResponseError> {
  let team = team.ok_or_else(|| ResponseError::NotFound("team not found".to_owned()))?;
  // team should check admin's message, so we should filter is_admin == true
  let chats = chat::get_unchecked(&db.conn, team.id, true).await?;
  Ok(Json(chats))
}
