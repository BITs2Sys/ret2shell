use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
  #[error("Failed to parse message: {0}")]
  ParseError(#[from] std::string::FromUtf8Error),
  #[error("Failed to parse message: {0}")]
  SerdeError(#[from] serde_json::Error),
}
