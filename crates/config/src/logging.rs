//! The configuration for logging in the application.
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::traits::Merge;

/// `LoggingConfig` represents the configuration for logging in the application.
#[derive(Clone, Debug, Serialize, Deserialize, FromJsonQueryResult, PartialEq, Eq)]
pub struct Config {
  /// `directory` is the path to the directory where log files will be stored.
  pub directory: String,
  /// `level` is the minimum log level that will be recorded (e.g., "info",
  /// "debug", "error").
  pub level: String,
  /// `files_kept` is the last n files that will be kept.
  pub files_kept: Option<usize>,
  /// compress files after they are rotated
  pub compress: Option<bool>,
}

impl Merge for Option<Config> {
  fn merge(self, _: Self) -> Self {
    // prefers return other if it is Some
    self
  }
}
