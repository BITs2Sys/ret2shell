use axum::Router;

use crate::controller::GlobalState;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
}
