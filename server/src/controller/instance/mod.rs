use axum::routing::get;
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Router};
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::controller::GlobalState;
use crate::entity::instance::get_user_current_instance;
use crate::entity::user::Model as UserModel;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new().route("/self", get(get_self_running_instance))
}

async fn get_self_running_instance(
    State(ref db): State<DatabaseConnection>,
    Extension(user): Extension<UserModel>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let instance = get_user_current_instance(db, user.id)
        .await
        .map_err(|err| {
            error!("get user current instance failed: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "encountered database error",
            )
        })?;
    Ok(Json(instance))
}
