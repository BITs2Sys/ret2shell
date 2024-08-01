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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthKeys {
  pub xdu: Option<OAuthKey>,
  pub xmu: Option<OAuthKey>,
  pub nwnu: Option<OAuthKey>,
  pub taru: Option<OAuthKey>,
  pub jiangnan: Option<OAuthKey>,
}

#[derive(Serialize, Deserialize, Clone, Debug, FromJsonQueryResult, PartialEq, Eq)]
pub struct Config {
  pub signing_key: String,
  pub buffer_time: i64,
  pub expires_time: i64,
  pub oauth_keys: Option<OAuthKeys>,
}

impl Config {
  pub fn desensitize(self) -> Self {
    Config {
      signing_key: "".to_owned(),
      oauth_keys: self.oauth_keys.map(|keys| OAuthKeys {
        xdu: keys.xdu.map(|key| key.desensitize()),
        xmu: keys.xmu.map(|key| key.desensitize()),
        nwnu: keys.nwnu.map(|key| key.desensitize()),
        taru: keys.taru.map(|key| key.desensitize()),
        jiangnan: keys.jiangnan.map(|key| key.desensitize()),
      }),
      ..self
    }
  }
}

impl Merge for Option<Config> {
  fn merge(self, other: Self) -> Self {
    // prefers fields in `other`
    match (self, other) {
      (Some(a), Some(b)) => Some(Config {
        signing_key: b.signing_key,
        buffer_time: b.buffer_time,
        expires_time: b.expires_time,
        oauth_keys: b.oauth_keys.or(a.oauth_keys),
      }),
      (Some(a), None) => Some(a),
      (None, Some(b)) => Some(b),
      (None, None) => None,
    }
  }
}
