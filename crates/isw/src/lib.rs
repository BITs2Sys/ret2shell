//! ISW (Internal Security Warfare) range mode for Ret2Shell — BITs2CTF fork.
//!
//! This crate is the platform-side manager + typed HTTP client for the per-host
//! `r2s-isw-agent`. It is intentionally decoupled from the database: the server
//! resolves host rows and passes `(address, port)` to [`IswManager::client_for`].

pub mod client;
pub mod manager;
pub mod protocol;

pub use client::AgentClient;
pub use manager::IswManager;

#[derive(Debug, thiserror::Error)]
pub enum IswError {
  #[error("http transport error: {0}")]
  Http(#[from] reqwest::Error),
  #[error("host-agent error: {0}")]
  Agent(String),
  #[error("configuration error: {0}")]
  Config(String),
}
