use std::collections::HashMap;

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuthError {
  #[error("missing oauth field: {0}")]
  MissingField(String),
  #[error("xml parse error: {0}")]
  XmlParseError(#[from] roxmltree::Error),
  #[error("network error: {0}")]
  NetworkError(#[from] reqwest::Error),
  #[error("adapter unavailable: {0}")]
  AdapterUnavailable(String),
  #[error("invalid email: {0}")]
  InvalidEmail(String),
}

#[async_trait::async_trait]
pub trait OAuthProvider {
  /// login to oauth account
  ///
  /// # returns
  ///
  /// * `Ok((auth_key, data))` - auth_key is the unique key for the oauth account, data is the user data
  async fn login(
    &self, account: &str, email: &str, query: HashMap<String, String>,
  ) -> Result<(String, Value), OAuthError>;
}
