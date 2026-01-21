use async_nats::jetstream::{Message, consumer::pull::Stream};
use chrono::{Duration, Utc};
use futures::StreamExt;
use r2s_event::{EventManager, events::EventContainer};
use r2s_migrator::Database;
use r2s_queue::TracedMessage;
use tracing::{error, error_span, warn};

use crate::traits::ResponseError;

pub fn spawn(messages: Stream, manager: EventManager, db: Database) {
  let saki = manager.clone();
  tokio::spawn(async move { saki.cry().await });
  tokio::spawn(event_pusher(messages, manager, db));
}

async fn event_pusher(mut messages: Stream, manager: EventManager, db: Database) {
  let mut retries = 0;
  loop {
    while let Some(message) = messages.next().await {
      if let Ok(message) = message {
        let result = push_event(message.clone(), manager.clone(), &db).await;
        if let Err(error) = result {
          error!(?error, "failed to process event message");
        }
        message.double_ack().await.ok();
      } else {
        error!(?message, "failed to receive event message from nats");
      }
    }
    retries += 1;
    if retries < 5 {
      warn!(
        "event pusher worker stopped unexpectedly! maybe a message queue issue? trying to restart..."
      );
      continue;
    } else {
      error!("event pusher worker stopped unexpectedly for 5 times, exiting...");
      break;
    }
  }
}

async fn push_event(
  message: Message, manager: EventManager, _db: &Database,
) -> Result<(), ResponseError> {
  let payload = String::from_utf8(message.message.payload.to_vec())?;
  let event = serde_json::from_str::<TracedMessage<EventContainer>>(&payload)?;
  let span = error_span!("request", trace=%event.trace);
  let span_guard = span.enter();
  if Utc::now().signed_duration_since(event.created_at) > Duration::minutes(10) {
    warn!("event message expired, dropping");
    message.double_ack().await.ok();
    return Ok(());
  }
  manager.broadcast(event.payload).await;
  drop(span_guard);
  Ok(())
}
