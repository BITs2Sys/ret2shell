use std::net::IpAddr;

use axum::{
  Extension, Router,
  extract::{Query, State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
};
use r2s_database::game;
use r2s_event::EventManager;
use r2s_migrator::Database;
use serde::Deserialize;
use tracing::{info, warn};

use crate::traits::{GlobalState, ResponseError};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route("/connect", get(connect_game))
}

#[derive(Deserialize)]
struct ConnectQuery {
  pub game_id: i64,
  pub client: Option<String>,
  pub token: String,
}

async fn connect_game(
  State(ref db): State<Database>, State(event): State<EventManager>,
  Extension(ip): Extension<IpAddr>,
  Query(ConnectQuery {
    game_id,
    token,
    client,
  }): Query<ConnectQuery>,
  ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ResponseError> {
  let game = game::get(&db.conn, game_id).await?;
  if let Some(game) = game
    && game.token.is_some_and(|t| t == token)
  {
    info!(
      client=%client.as_deref().unwrap_or("Unspecified v0.0.0"),
      %game_id,
      %ip,
      "game event pusher connection established for game",
    );
    return Ok(ws.on_upgrade(move |ws| async move {
      event
        .subscribe(
          game_id,
          ip,
          client.unwrap_or("Unspecified v0.0.0".to_owned()),
          ws,
        )
        .await;
    }));
  }
  warn!(
    %game_id,
    %ip,
    "game event pusher connection attempt with invalid token for game",
  );
  Err(ResponseError::Forbidden("permission denied".to_owned()))
}
