use async_nats::jetstream::consumer::pull::Stream;
use axum::extract::ws::{Message, WebSocket};
use events::EventContainer;
use tokio::sync::broadcast;

pub mod events;
mod worker;
pub use events::Event;
pub mod traits;
pub use traits::EventError;

#[derive(Clone, Debug)]
pub struct EventManager {
  tx: broadcast::Sender<EventContainer>,
}

impl Default for EventManager {
  fn default() -> Self {
    Self::new()
  }
}

impl EventManager {
  pub fn new() -> Self {
    let (tx, _) = broadcast::channel(128);
    Self { tx }
  }

  pub async fn subscribe(&self, id: i64, mut ws: WebSocket) {
    let mut rx = self.tx.subscribe();

    while let Ok(event) = rx.recv().await {
      if event.game_id != id {
        continue;
      }
      let message = serde_json::to_string(&event.event).unwrap();
      ws.send(Message::Text(message)).await.ok();
    }
  }

  pub async fn broadcast(&self, message: EventContainer) {
    self.tx.send(message).ok();
  }
}

pub async fn initialize(stream: Stream) -> EventManager {
  let manager = EventManager::new();
  let future = worker::event_pusher(stream, manager.clone());
  tokio::spawn(future);
  manager
}
