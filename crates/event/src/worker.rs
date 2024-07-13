use async_nats::jetstream::{consumer::pull::Stream, Message};
use futures::StreamExt;
use tracing::error;

use crate::{events::EventContainer, traits::EventError, EventManager};

pub async fn event_pusher(mut messages: Stream, manager: EventManager) {
  while let Some(message) = messages.next().await {
    if let Ok(message) = message {
      let result = push_event(message.clone(), manager.clone()).await;
      if let Err(error) = result {
        error!("Failed to process message: {:?}", error);
      }
      message.ack().await.ok();
    } else {
      error!("Failed to receive message from nats: {:?}", message);
    }
  }
}

async fn push_event(message: Message, manager: EventManager) -> Result<(), EventError> {
  let payload = String::from_utf8(message.message.payload.to_vec())?;
  let event = serde_json::from_str::<EventContainer>(&payload)?;
  manager.broadcast(event).await;
  Ok(())
}
