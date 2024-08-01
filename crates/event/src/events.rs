use r2s_database::{challenge, submission, team, user};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeEventType {
  Up,
  Down,
  NewHint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChallengeEvent {
  pub challenge: challenge::Model,
  pub operator: user::Model,
  pub event_type: ChallengeEventType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionEventType {
  Correct,
  Cheated,
  TooManySubmits,
  TooQuick,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubmissionEvent {
  pub submission: submission::Model,
  pub blood_state: Option<u64>,
  pub operator: user::Model,
  pub team: team::Model,
  pub peer_team: Option<team::Model>,
  pub reason: Option<String>,
  pub event_type: SubmissionEventType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameEventType {
  Freeze,
  Unfreeze,
  NewNotification,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameEvent {
  pub operator: user::Model,
  pub event_type: GameEventType,
  pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatEventType {
  Message,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatEvent {
  pub operator: user::Model,
  pub team: team::Model,
  pub challenge: challenge::Model,
  pub event_type: ChatEventType,
  pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DevopsEventType {
  ClusterOverloaded,
  ClusterRecovered,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DevopsEvent {
  pub event_type: DevopsEventType,
  pub running: i64,
  pub pending: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
  Challenge(ChallengeEvent),
  Submission(SubmissionEvent),
  Game(GameEvent),
  Chat(ChatEvent),
  Devops(DevopsEvent),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventContainer {
  pub game_id: i64,
  pub event: Event,
}
