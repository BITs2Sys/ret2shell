use super::GlobalState;
use axum::Router;
use std::sync::Arc;

mod answer;
mod repo;
mod submission;
mod workflow;
mod traffic;

pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
}
