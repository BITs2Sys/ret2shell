use crate::controller::GlobalState;
use axum::Router;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
}
