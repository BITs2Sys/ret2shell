use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::traits::Merge;

#[derive(Clone, Debug, Default, Serialize, Deserialize, FromJsonQueryResult, PartialEq, Eq)]
pub struct Config {
  /// Whether email functionality is enabled or not.
  pub enabled: bool,
  /// The email server host.
  pub host: String,
  /// The email server port.
  pub port: u16,
  /// The name used for the sender.
  pub sender: String,
  /// The email address used as the sender.
  pub sender_address: Option<String>,
  /// The username for authentication with the email server.
  pub username: String,
  /// The password for authentication with the email server.
  pub password: String,
  /// The TLS configuration for secure email communication.
  /// could be "none", "tls", "starttls"
  pub tls: String,
  /// The email body for reset password emails
  pub reset_password_email_body: Option<String>,
  /// The email subject for reset password emails.
  pub reset_password_email_subject: Option<String>,
  /// The email body for email verification emails.
  pub verify_email_body: Option<String>,
  /// The email subject for email verification emails.
  pub verify_email_subject: Option<String>,
}

impl Merge for Option<Config> {
  fn merge(self, other: Self) -> Self {
    // prefers fields in `other`
    match (self, other) {
      (_, Some(b)) => Some(b),
      (Some(a), None) => Some(a),
      (None, None) => None,
    }
  }
}
