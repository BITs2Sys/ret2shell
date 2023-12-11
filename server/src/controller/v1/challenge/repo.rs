use axum::Router;

use crate::controller::GlobalState;

/*
 * Repo router
 *
 * user does not have any permissions, so this router if fully under admin
 * layer.
 */
pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new()
}
