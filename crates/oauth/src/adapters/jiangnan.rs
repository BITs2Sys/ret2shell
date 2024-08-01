use std::collections::HashMap;

use r2s_config::auth::OAuthKey;
use serde_json::Value;

use crate::traits::{OAuthError, OAuthProvider as OAuthProviderTrait};

#[derive(Clone, Debug)]
pub struct OAuthProvider {
  pub key: OAuthKey,
}

#[async_trait::async_trait]
impl OAuthProviderTrait for OAuthProvider {
  async fn login(
    &self, _account: &str, email: &str, _query: HashMap<String, String>,
  ) -> Result<(String, Value), OAuthError> {
    if email.ends_with("jiangnan.edu.cn") {
      Ok((email.to_string(), serde_json::json!({ "email": email })))
    } else {
      Err(OAuthError::InvalidEmail(email.to_string()))
    }
  }
}
