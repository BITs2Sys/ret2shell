use std::collections::HashMap;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::traits::Merge;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthKey {
  pub id: String,
  pub key: String,
}

impl OAuthKey {
  pub fn desensitize(self) -> Self {
    OAuthKey {
      key: "".to_owned(),
      ..self
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, FromJsonQueryResult, PartialEq, Eq)]
pub struct Config {
  pub signing_key: String,
  pub buffer_time: i64,
  pub expires_time: i64,
  pub oauth_keys: HashMap<String, OAuthKey>,
}

impl Config {
  pub fn desensitize(self) -> Self {
    Config {
      signing_key: "".to_owned(),
      oauth_keys: self
        .oauth_keys
        .into_iter()
        .map(|(k, v)| (k, v.desensitize()))
        .collect(),
      ..self
    }
  }
}

impl Merge for Option<Config> {
  fn merge(self, other: Self) -> Self {
    // prefers fields in `other`
    match (self, other) {
      (Some(a), _) => Some(a),
      (None, Some(b)) => Some(b),
      (None, None) => None,
    }
  }
}
