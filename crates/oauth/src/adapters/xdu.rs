use std::collections::HashMap;

use r2s_config::auth::OAuthKey;
use serde_json::Value;

use crate::{
  traits::{OAuthError, OAuthProvider as OAuthProviderTrait},
  utility::cas::get_user_info_by_ticket,
};

#[derive(Clone, Debug)]
pub struct OAuthProvider {
  pub key: OAuthKey,
}

#[async_trait::async_trait]
impl OAuthProviderTrait for OAuthProvider {
  async fn login(
    &self, _account: &str, _email: &str, query: HashMap<String, String>,
  ) -> Result<(String, Value), OAuthError> {
    let ticket = query
      .get("ticket")
      .ok_or(OAuthError::MissingField("ticket".to_string()))?;
    let resp = get_user_info_by_ticket(
      "https://ids.xidian.edu.cn/authserver/serviceValidate",
      &self.key.id.clone(),
      ticket,
    )
    .await?;
    Ok((
      resp.id.clone(),
      serde_json::json!({ "name": resp.name, "id": resp.id }),
    ))
  }
}
