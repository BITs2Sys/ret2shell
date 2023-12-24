//! Automate
//!
//! this module provides API for automation uses, such as platform sync,
//! platform export, etc.

use axum::Router;

use crate::controller::GlobalState;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
}
