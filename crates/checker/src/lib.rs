use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use r2s_bucket::challenge::ChallengeBucket;
use r2s_database::{submission, team, user};
use rune::{
  alloc,
  runtime::{Object, RuntimeContext},
  Context, Source, Sources, Unit, Value, Vm,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error};
use traits::CheckerError;

pub mod traits;

type CheckerContext = (Arc<Unit>, Arc<RuntimeContext>, DateTime<Utc>);

#[derive(Clone, Debug)]
pub struct Checker {
  contexts: Arc<RwLock<HashMap<String, CheckerContext>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditMessage {
  peer_team: Option<i64>,
  reason: String,
}

macro_rules! to_rune_object {
    ($model:tt, $($column:tt), *) => {
        {
            let mut object = Object::new();
            $(
                object.insert(alloc::String::try_from(stringify!($column))?, rune::to_value($model.$column)?)?;
            )*
            object
        }
    };
}

impl Default for Checker {
  fn default() -> Self {
    Self::new()
  }
}

impl Checker {
  pub fn new() -> Self {
    Self {
      contexts: Arc::new(RwLock::new(HashMap::new())),
    }
  }
  pub async fn preload(&mut self, bucket: ChallengeBucket) -> Result<(), CheckerError> {
    let script_folder = bucket.path().to_path_buf().join("checker");
    if !script_folder.exists() {
      error!(
        "Missing checker script: {}: {}",
        bucket.name,
        script_folder.display()
      );
      Err(CheckerError::MissingCheckerScript(format!(
        "{}: {}",
        bucket.name,
        script_folder.display()
      )))
    } else {
      let key = bucket.path().to_path_buf().display().to_string();
      if self.contexts.read().await.contains_key(key.as_str()) {
        return Ok(());
      }
      let mut context = Context::with_default_modules()?;
      context.install(rune_modules::http::module(true)?)?;
      context.install(rune_modules::json::module(true)?)?;
      context.install(rune_modules::toml::module(true)?)?;
      context.install(rune_modules::process::module(true)?)?;
      context.install(ret2script::modules::crypto::module(true)?)?;
      context.install(ret2script::modules::bucket::module(true)?)?;
      context.install(ret2script::modules::audit::module(true)?)?;
      context.install(ret2script::modules::utils::module(true)?)?;
      context.install(ret2script::modules::regex::module(true)?)?;
      let mut sources = Sources::new();
      sources.insert(Source::from_path(script_folder.join("main.rx"))?)?;
      debug!("Preloading checker script: {}", script_folder.display());

      let unit = rune::prepare(&mut sources).with_context(&context).build()?;
      let runtime = context.runtime()?;

      self.contexts.write().await.insert(
        bucket.name.clone(),
        (Arc::new(unit), Arc::new(runtime), Utc::now()),
      );
      Ok(())
    }
  }

  /// Check the flag and return results and audit messages.
  ///
  /// ## Returns
  ///
  /// (correct: bool, msg: String, Option<(peer_team: Option<i64>, reason:
  /// String)>)
  pub async fn check(
    &self, bucket: ChallengeBucket, user: user::Model, team: Option<team::Model>,
    submission: submission::Model,
  ) -> Result<(bool, String, Option<AuditMessage>), CheckerError> {
    let contexts = self.contexts.read().await;
    let (unit, runtime, _) = contexts
      .get(bucket.name.as_str())
      .ok_or(CheckerError::MissingCheckerScript(bucket.name.clone()))?;
    let mut vm = Vm::new(runtime.clone(), unit.clone());
    let user_object = to_rune_object!(user, id, account, institute_id);
    let submission_object =
      to_rune_object!(submission, id, user_id, team_id, challenge_id, content);
    let team_object = match team {
      Some(team) => to_rune_object!(team, id, name, institute_id),
      None => Object::new(),
    };
    let bucket_path = alloc::String::try_from(bucket.path().to_path_buf().display().to_string())?;
    let output = vm
      .async_call(
        ["check"],
        (bucket_path, user_object, team_object, submission_object),
      )
      .await?;
    let output: Result<(bool, String, Option<Object>), Value> = rune::from_value(output)?;
    if let Ok((result, message, audit)) = output {
      let audit = if let Some(audit) = audit {
        Some(AuditMessage {
          peer_team: rune::from_value(
            audit
              .get("peer_team")
              .ok_or(CheckerError::MissingResultField(
                "audit::peer_team".to_owned(),
              ))?
              .to_owned(),
          )?,
          reason: rune::from_value(
            audit
              .get("reason")
              .ok_or(CheckerError::MissingResultField(
                "audit::peer_team".to_owned(),
              ))?
              .to_owned(),
          )?,
        })
      } else {
        None
      };
      Ok((result, message, audit))
    } else {
      Err(CheckerError::ScriptError(
        "Early returns from script".to_owned(),
      ))
    }
  }

  pub async fn environ(
    &self, bucket: ChallengeBucket, user: user::Model, team: Option<team::Model>,
  ) -> Result<HashMap<String, String>, CheckerError> {
    let contexts = self.contexts.read().await;
    let (unit, runtime, _) = contexts
      .get(bucket.name.as_str())
      .ok_or(CheckerError::MissingCheckerScript(bucket.name.clone()))?;
    let mut vm = Vm::new(runtime.clone(), unit.clone());
    let user_object = to_rune_object!(user, id, account, institute_id);
    let team_object = match team {
      Some(team) => to_rune_object!(team, id, name, institute_id),
      None => Object::new(),
    };
    let bucket_path = alloc::String::try_from(bucket.path().to_path_buf().display().to_string())?;
    let output = vm
      .async_call(["environ"], (bucket_path, user_object, team_object))
      .await?;
    let object: Result<Object, Value> = rune::from_value(output)?;
    if let Ok(object) = object {
      let mut environ = HashMap::new();
      for (key, value) in object.iter() {
        environ.insert(key.to_string(), rune::from_value(value.clone())?);
      }
      Ok(environ)
    } else {
      Err(CheckerError::ScriptError(
        "Early returns from script".to_owned(),
      ))
    }
  }

  pub async fn cleanup(&mut self) {
    let now = Utc::now();
    self.contexts.write().await.retain(|_, (_, _, time)| {
      let duration = now.signed_duration_since(*time);
      duration.num_hours() < 1
    });
  }

  pub async fn cleanup_worker(&mut self) {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(15 * 60)).await;
      tracing::debug!("Running checker cleanup...");
      self.cleanup().await;
      tracing::trace!("Live checkers: {:?}", self.contexts.read().await.keys());
    }
  }
}

pub async fn initialize() -> Checker {
  let checker = Checker::new();
  let mut checker_worker = checker.clone();
  tokio::spawn(async move { checker_worker.cleanup_worker().await });
  checker
}
