use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use r2s_bucket::challenge::ChallengeBucket;
use r2s_database::{challenge, submission, team, user};
use rune::{
  alloc,
  runtime::{Object, RuntimeContext},
  termcolor::Buffer,
  Context, Diagnostics, Source, Sources, Unit, Value, Vm,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::debug;
use traits::CheckerError;

pub mod traits;

type CheckerContext = (Arc<Unit>, Arc<RuntimeContext>, DateTime<Utc>);

#[derive(Clone, Debug)]
pub struct Checker {
  contexts: Arc<RwLock<HashMap<String, CheckerContext>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditMessage {
  pub peer_team: Option<i64>,
  pub reason: String,
}

macro_rules! to_rune_object {
    ($model:tt, $($column:tt), *) => {
        {
            let mut object = Object::new();
            $(
                object.insert(alloc::String::try_from(stringify!($column))?, rune::to_value($model.$column.clone())?)?;
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

  async fn context() -> Result<Context, CheckerError> {
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
    Ok(context)
  }

  async fn sources(bucket: &ChallengeBucket) -> Result<Sources, CheckerError> {
    let mut sources = Sources::new();
    sources.insert(Source::memory(
      bucket
        .checker()
        .await
        .map_err(|e| CheckerError::MissingCheckerScript(e.to_string()))?,
    )?)?;
    Ok(sources)
  }

  pub async fn lint(&self, bucket: &ChallengeBucket) -> Result<(), CheckerError> {
    let context = Self::context().await?;
    let mut sources = Self::sources(bucket).await?;
    let mut diagnostics = Diagnostics::new();
    let _ = rune::prepare(&mut sources)
      .with_context(&context)
      .with_diagnostics(&mut diagnostics)
      .build();
    if !diagnostics.is_empty() {
      let mut out = Buffer::ansi();
      diagnostics.emit(&mut out, &sources)?;
      return Err(CheckerError::CompileError((String::from_utf8(out.into_inner())?).to_string()));
    }
    let unit = rune::prepare(&mut sources).with_context(&context).build()?;
    let runtime = context.runtime()?;
    let vm = Vm::new(Arc::new(runtime), Arc::new(unit));
    vm.lookup_function(["check"])
      .map_err(|_| CheckerError::MissingFunction("check".to_owned()))?;
    vm.lookup_function(["environ"])
      .map_err(|_| CheckerError::MissingFunction("environ".to_owned()))?;
    Ok(())
  }

  pub async fn expire(&mut self, bucket: &ChallengeBucket) {
    self.contexts.write().await.remove(&bucket.hash());
  }

  pub async fn preload(
    &mut self, challenge: &challenge::Model, bucket: &ChallengeBucket,
  ) -> Result<(), CheckerError> {
    let mut contexts = self.contexts.write().await;
    if contexts.contains_key(&bucket.hash()) && challenge.updated_at < contexts[&bucket.hash()].2 {
      return Ok(());
    }
    let context = Self::context().await?;
    let mut sources = Self::sources(bucket).await?;

    let unit = rune::prepare(&mut sources).with_context(&context).build()?;
    let runtime = context.runtime()?;

    contexts.insert(
      bucket.hash(),
      (Arc::new(unit), Arc::new(runtime), Utc::now()),
    );
    Ok(())
  }

  /// Check the flag and return results and audit messages.
  ///
  /// ## Returns
  ///
  /// (correct: bool, msg: String, Option<(peer_team: Option<i64>, reason:
  /// String)>)
  pub async fn check(
    &self, bucket: &ChallengeBucket, user: &user::Model, team: &Option<team::Model>,
    submission: &submission::Model,
  ) -> Result<(bool, String, Option<AuditMessage>), CheckerError> {
    let contexts = self.contexts.read().await;
    let (unit, runtime, _) = contexts
      .get(&bucket.hash())
      .ok_or(CheckerError::MissingCheckerScript(bucket.name.clone()))?;
    let mut vm = Vm::new(runtime.clone(), unit.clone());
    let user_object = to_rune_object!(user, id, account, institute_id);
    let submission_object =
      to_rune_object!(submission, id, user_id, team_id, challenge_id, content);
    let team_object = match team {
      Some(team) => to_rune_object!(team, id, name, institute_id),
      None => Object::new(),
    };
    let bucket = ret2script::modules::bucket::Bucket::try_new(bucket.path())?;
    let output = vm.call(
      ["check"],
      (bucket, user_object, team_object, submission_object),
    )?;
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
    &self, bucket: &ChallengeBucket, user: &user::Model, team: &Option<team::Model>,
  ) -> Result<HashMap<String, String>, CheckerError> {
    debug!("entering checker environ");
    let contexts = self.contexts.read().await;
    let (unit, runtime, _) = contexts
      .get(&bucket.hash())
      .ok_or(CheckerError::MissingCheckerScript(bucket.name.clone()))?;
    let mut vm = Vm::new(runtime.clone(), unit.clone());
    let user_object = to_rune_object!(user, id, account, institute_id);
    let team_object = match team {
      Some(team) => to_rune_object!(team, id, name, institute_id),
      None => Object::new(),
    };
    let bucket = ret2script::modules::bucket::Bucket::try_new(bucket.path())?;
    debug!("calling environ");
    let output = vm.call(["environ"], (bucket, user_object, team_object))?;
    debug!("environ output: {:?}", output);
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
