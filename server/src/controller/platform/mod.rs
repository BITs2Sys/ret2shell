use axum::Router;
use std::sync::Arc;

use crate::controller::GlobalState;

pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
}
