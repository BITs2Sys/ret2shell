use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use sea_orm::DatabaseConnection;
use tracing::error;

use crate::{controller::GlobalState, entity::institute};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
    Router::new().route("/", get(get_institute_list))
}

async fn get_institute_list(
    State(ref conn): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    Ok(Json(institute::get_institute_list(conn).await.map_err(
        |e| {
            error!("Failed to get institute list: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get institute list",
            )
        },
    )?))
}
