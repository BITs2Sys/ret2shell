use axum::{
  extract::{Path, Query, State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
  Router,
};
use r2s_cluster::Cluster;
use serde::Deserialize;
use tracing::error;

use crate::traits::{GlobalState, ResponseError};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route("/:token", get(link_challenge_env))
}

#[derive(Deserialize)]
struct LinkQuery {
  port: u16,
}

async fn link_challenge_env(
  State(cluster): State<Cluster>, Path(token): Path<String>,
  Query(LinkQuery { port }): Query<LinkQuery>, ws: Option<WebSocketUpgrade>,
) -> Result<impl IntoResponse, ResponseError> {
  if ws.is_none() {
    return Err(ResponseError::BadRequest("WebSocket required".to_string()));
  }
  let ws = ws.unwrap();
  Ok(ws.on_upgrade(move |socket| async move {
    let result = cluster
      .at("ret2shell-challenge")
      .wsrx_link(&token, port, socket)
      .await;
    if let Err(e) = result {
      error!("Failed to link challenge env: {:?}", e);
    }
  }))
}
