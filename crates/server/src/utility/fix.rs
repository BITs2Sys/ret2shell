use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const FIX_SUBMISSION_PREFIX: &str = "__r2s_fix__:";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixSubmissionMeta {
  pub token: String,
  pub file_name: String,
}

pub fn encode_fix_submission_meta(meta: &FixSubmissionMeta) -> Result<String, serde_json::Error> {
  Ok(format!(
    "{}{}",
    FIX_SUBMISSION_PREFIX,
    serde_json::to_string(meta)?
  ))
}

pub fn decode_fix_submission_meta(content: Option<&str>) -> Option<FixSubmissionMeta> {
  let payload = content?.strip_prefix(FIX_SUBMISSION_PREFIX)?;
  serde_json::from_str(payload).ok()
}

pub fn is_fix_submission(content: Option<&str>) -> bool {
  content.is_some_and(|content| content.starts_with(FIX_SUBMISSION_PREFIX))
}

pub fn fix_upload_dir(token: &str) -> PathBuf {
  std::env::temp_dir()
    .join("ret2shell")
    .join("fix")
    .join(token)
}

pub fn fix_upload_path(token: &str) -> PathBuf {
  fix_upload_dir(token).join("upload")
}

pub fn shell_quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn parse_fix_result(logs: &str, env_name: &str) -> Option<String> {
  let prefix = format!("{env_name}=");
  logs
    .lines()
    .rev()
    .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
    .map(ToOwned::to_owned)
}
