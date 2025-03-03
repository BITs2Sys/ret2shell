use axum::{
  Router,
  extract::{Path, Query, State, WebSocketUpgrade},
  response::IntoResponse,
  routing::get,
};
use r2s_cluster::{CHALLENGE_NS, Cluster};
use serde::Deserialize;
use tracing::debug;

use crate::traits::{GlobalState, ResponseError};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new().route(
    "/{token}",
    get(link_challenge_env).options(ping_challenge_env),
  )
}

#[derive(Deserialize)]
struct LinkQuery {
  port: u16,
}

async fn link_challenge_env(
  State(cluster): State<Cluster>, Path(token): Path<String>,
  Query(LinkQuery { port }): Query<LinkQuery>, ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(ws.on_upgrade(move |socket| async move {
    let result = cluster
      .at(CHALLENGE_NS)
      .wsrx_link(&token, port, socket)
      .await;
    if let Err(e) = result {
      debug!("Failed to link challenge env: {:?}", e);
    }
  }))
}

async fn ping_challenge_env() -> impl IntoResponse {
  "pong"
}
