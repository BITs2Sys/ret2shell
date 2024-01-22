use std::collections::HashMap;

use crate::entity::challenge;
use crate::entity::submission;
use crate::entity::user;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CheckerError {}

pub trait FlagChecker {
    async fn check(
        &self, user: &user::Model, challenge: &challenge::Model, submission: &submission::Model,
    ) -> Result<bool, CheckerError>;
    async fn flag(
        &self, user: &user::Model, challenge: &challenge::Model,
    ) -> Result<String, CheckerError>;
    async fn env_vars(
        &self, user: &user::Model, challenge: &challenge::Model,
    ) -> Result<HashMap<String, String>, CheckerError>;
}
