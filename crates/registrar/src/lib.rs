use r2s_engine::{DiagnosticMarker, Engine, EngineError, GLOBAL_ENGINE};
use rune::{Any, ContextError, Module, runtime::Object};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Any)]
#[rune(item = ::ret2shell::registrar)]
pub struct RegisterInfo {
  pub account: String,
  pub nickname: String,
  pub email: String,
}

#[derive(Clone, Debug, Default, Any, Serialize, Deserialize)]
#[rune(item = ::ret2shell::registrar)]
pub struct RegisterResult {
  /// When true, skip sending verification email and mark verified
  pub bypass_email_verification: bool,
  /// Optionally assign an institute id directly
  pub institute_id: Option<i64>,
  /// Or join institute by token
  pub institute_token: Option<String>,
}

#[derive(Error, Debug, Any)]
#[rune(item = ::ret2shell::registrar)]
pub enum RegistrarError {
  #[error("rejected: {0}")]
  Rejected(String),
  #[error("engine error: {0}")]
  EngineError(#[from] EngineError),
}

#[rune::module(::ret2shell::registrar)]
pub fn module(_stdio: bool) -> Result<Module, ContextError> {
  let mut module = Module::from_meta(self::module_meta)?;
  module.ty::<RegisterInfo>()?;
  module.ty::<RegisterResult>()?;
  Ok(module)
}

#[derive(Clone, Debug, Default)]
pub struct Registrar;

impl Registrar {
  fn default_modules() -> Vec<fn(bool) -> Result<rune::Module, rune::ContextError>> {
    vec![
      rune_modules::json::module,
      rune_modules::toml::module,
      module,
    ]
  }

  pub async fn lint(&self, script: impl AsRef<str>) -> Result<Vec<DiagnosticMarker>, EngineError> {
    Engine::lint(Self::default_modules(), script, &["intercept"]).await
  }

  pub async fn preload(
    &self, key: impl AsRef<str>, script: impl AsRef<str>,
  ) -> Result<(), EngineError> {
    let key = format!("registrar-{}", key.as_ref());
    GLOBAL_ENGINE
      .preload(Self::default_modules(), key, script, None)
      .await
  }

  pub async fn intercept(
    &self, key: impl AsRef<str>, info: &RegisterInfo,
  ) -> Result<RegisterResult, RegistrarError> {
    let key = format!("registrar-{}", key.as_ref());
    let result = GLOBAL_ENGINE
      .execute(key, "intercept", (info.clone(),))
      .await?;
    // Expect either an object with fields or a string error
    if let Ok(object) = rune::from_value::<Object>(result.clone()) {
      let mut resp = RegisterResult::default();
      for (k, v) in object.iter() {
        match k.as_str() {
          "bypass_email_verification" => {
            resp.bypass_email_verification = rune::from_value(v.clone()).unwrap_or(false);
          }
          "institute_id" => {
            resp.institute_id = rune::from_value(v.clone()).ok();
          }
          "institute_token" => {
            resp.institute_token = rune::from_value(v.clone()).ok();
          }
          _ => {}
        }
      }
      Ok(resp)
    } else if let Ok(reason) = rune::from_value::<String>(result) {
      Err(RegistrarError::Rejected(reason))
    } else {
      Err(RegistrarError::Rejected(
        "invalid registrar result".to_owned(),
      ))
    }
  }
}
