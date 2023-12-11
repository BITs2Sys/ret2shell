//! Sensitive content audit function, which is used to audit the user/team's
//! name, email, etc.
//!
//! > ATTENTION: Please note that the function implemented here is very strict
//! > in checking sensitive content
//! and its performance has some problems, so please do not block users directly
//! based on the results returned here when using it, but add it to the manual
//! review queue.
//!
//! We do not provide the sensitive word list here, please prepare it yourself,
//! e.g. <https://github.com/cjh0613/strict-sensitive-word>

use aho_corasick::AhoCorasick;

use crate::config::GlobalConfig;

pub mod word_filter;

#[derive(Clone)]
pub struct Auditor {
    /// The `AhoCorasick` word filter used by the `Auditor` to filter out
    /// unwanted words.
    pub word_filter: AhoCorasick,
}

/// Initializes an `Auditor` instance with the given configuration.
pub async fn initialize(config: &GlobalConfig) -> anyhow::Result<Auditor> {
    let word_filter = word_filter::initialize(config).await?;
    Ok(Auditor { word_filter })
}
