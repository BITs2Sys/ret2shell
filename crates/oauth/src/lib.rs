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
  pub uestc_email: Option<adapters::uestc_email::OAuthProvider>,
  pub seu_email: Option<adapters::seu_email::OAuthProvider>,
  pub fudan_email: Option<adapters::fudan_email::OAuthProvider>,
  pub jlu_email: Option<adapters::jlu_email::OAuthProvider>,
  pub bit_email: Option<adapters::bit_email::OAuthProvider>,
  pub tzc_email: Option<adapters::tzc_email::OAuthProvider>,
  pub cppu_email: Option<adapters::cppu_email::OAuthProvider>,
}

macro_rules! map_oauth_providers {
  ($self:ident, $ori:ident, $($provider:ident), *) => {
    match $ori {
      $(
        stringify!($provider) => $self
          .$provider
          .as_ref()
          .map(|x| x as &dyn traits::OAuthProvider),
      )*
      _ => None,
    }
  }
}

macro_rules! gen_config {
  ($config:ident, $provider:ident) => {
    $config
      .oauth_keys
      .get(stringify!($provider))
      .as_ref()
      .map(|key| adapters::$provider::OAuthProvider {
        key: key.to_owned().clone(),
      })
  };
}

impl OAuth {
  pub fn from_config(config: &Config) -> Self {
    OAuth {
      xdu_cas: gen_config!(config, xdu_cas),
      xmu_cas: gen_config!(config, xmu_cas),
      jiangnan_email: gen_config!(config, jiangnan_email),
      hdu_email: gen_config!(config, hdu_email),
      cumt_email: gen_config!(config, cumt_email),
      uestc_email: gen_config!(config, uestc_email),
      seu_email: gen_config!(config, seu_email),
      fudan_email: gen_config!(config, fudan_email),
      jlu_email: gen_config!(config, jlu_email),
      bit_email: gen_config!(config, bit_email),
      tzc_email: gen_config!(config, tzc_email),
      cppu_email: gen_config!(config, cppu_email),
    }
  }

  pub fn get_provider(&self, provider: &str) -> Option<&dyn traits::OAuthProvider> {
    map_oauth_providers!(
      self,
      provider,
      xdu_cas,
      xmu_cas,
      jiangnan_email,
      hdu_email,
      cumt_email,
      uestc_email,
      seu_email,
      fudan_email,
      jlu_email,
      bit_email,
      tzc_email,
      cppu_email
    )
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
      uestc_email: None,
      seu_email: None,
      fudan_email: None,
      jlu_email: None,
      bit_email: None,
      tzc_email: None,
      cppu_email: None,
    }
  }
}
