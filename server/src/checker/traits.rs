use std::collections::HashMap;

use thiserror::Error;

use crate::entity::{challenge, submission, user};

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
