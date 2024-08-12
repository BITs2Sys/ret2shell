pub mod adapters;
pub mod traits;
mod utility;
use r2s_config::auth::Config;
pub use traits::OAuthError;

#[derive(Clone, Debug)]
pub struct OAuth {
  pub xdu_cas: Option<adapters::xdu_cas::OAuthProvider>,
  pub xmu_cas: Option<adapters::xmu_cas::OAuthProvider>,
  pub jiangnan_email: Option<adapters::jiangnan_email::OAuthProvider>,
  pub hdu_email: Option<adapters::hdu_email::OAuthProvider>,
  pub cumt_email: Option<adapters::cumt_email::OAuthProvider>,
}

impl OAuth {
  pub fn from_config(config: &Config) -> Self {
    OAuth {
      xdu_cas: config.oauth_keys.get("xdu_cas").as_ref().map(|key| {
        adapters::xdu_cas::OAuthProvider {
          key: key.to_owned().clone(),
        }
      }),
      xmu_cas: config.oauth_keys.get("xmu_cas").as_ref().map(|key| {
        adapters::xmu_cas::OAuthProvider {
          key: key.to_owned().clone(),
        }
      }),
      jiangnan_email: config.oauth_keys.get("jiangnan_email").as_ref().map(|key| {
        adapters::jiangnan_email::OAuthProvider {
          key: key.to_owned().clone(),
        }
      }),
      hdu_email: config.oauth_keys.get("hdu_email").as_ref().map(|key| {
        adapters::hdu_email::OAuthProvider {
          key: key.to_owned().clone(),
        }
      }),
      cumt_email: config.oauth_keys.get("cumt_email").as_ref().map(|key| {
        adapters::cumt_email::OAuthProvider {
          key: key.to_owned().clone(),
        }
      }),
    }
  }

  pub fn get_provider(&self, provider: &str) -> Option<&dyn traits::OAuthProvider> {
    match provider {
      "xdu_cas" => self
        .xdu_cas
        .as_ref()
        .map(|xdu| xdu as &dyn traits::OAuthProvider),
      "xmu_cas" => self
        .xmu_cas
        .as_ref()
        .map(|xmu| xmu as &dyn traits::OAuthProvider),
      "jiangnan_email" => self
        .jiangnan_email
        .as_ref()
        .map(|jiangnan| jiangnan as &dyn traits::OAuthProvider),
      "hdu_email" => self
        .hdu_email
        .as_ref()
        .map(|hdu| hdu as &dyn traits::OAuthProvider),
      "cumt_email" => self
        .cumt_email
        .as_ref()
        .map(|cumt| cumt as &dyn traits::OAuthProvider),
      _ => None,
    }
  }
}

pub async fn initialize(config: &Option<Config>) -> OAuth {
  if let Some(config) = config {
    OAuth::from_config(config)
  } else {
    OAuth {
      xdu_cas: None,
      xmu_cas: None,
      jiangnan_email: None,
      hdu_email: None,
      cumt_email: None,
    }
  }
}
