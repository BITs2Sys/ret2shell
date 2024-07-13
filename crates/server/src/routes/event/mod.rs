use axum::{
  extract::{Query, State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
  Router,
};
use r2s_database::game;
use r2s_event::EventManager;
use r2s_migrator::Database;
use serde::Deserialize;

use crate::traits::{GlobalState, ResponseError};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route("/connect", get(connect_game))
}

#[derive(Deserialize)]
struct ConnectQuery {
  pub game_id: i64,
  pub token: String,
}

async fn connect_game(
  State(ref db): State<Database>, State(event): State<EventManager>,
  Query(ConnectQuery { game_id, token }): Query<ConnectQuery>, ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ResponseError> {
  let game = game::get(&db.conn, game_id).await?;
  if let Some(game) = game {
    if game.token.is_some_and(|t| t == token) {
      return Ok(ws.on_upgrade(move |ws| async move {
        event.subscribe(game_id, ws).await;
      }));
    }
  }
  Err(ResponseError::Forbidden(
    "permission denied".to_owned(),
    format!(
      "event api was called with invalid token for game {}",
      game_id
    ),
  ))
}
