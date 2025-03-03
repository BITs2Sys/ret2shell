use axum::{
  Extension, Json, Router,
  extract::State,
  middleware,
  response::IntoResponse,
  routing::{get, post},
};
use r2s_database::{institute, user::Permission};
use r2s_migrator::Database;

use crate::{
  middleware::{auth, data},
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route(
      "/{institute}",
      get(get_institute)
        .patch(update_institute)
        .delete(delete_institute),
    )
    .route_layer(middleware::from_fn_with_state(
      state.clone(),
      data::prepare_data!(institute, false),
    ))
    .route("/", post(create_institute))
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::User
    )))
    .route("/", get(get_institute_list))
}

async fn get_institute_list(
  State(ref db): State<Database>, Extension(token): Extension<auth::Token>,
) -> Result<impl IntoResponse, ResponseError> {
  let institutes = institute::get_list(&db.conn).await?;
  if token.permissions.0.contains(&Permission::User) {
    Ok(Json(institutes))
  } else {
    Ok(Json(
      institutes
        .into_iter()
        .map(|institute| institute.desensitize())
        .collect(),
    ))
  }
}

async fn create_institute(
  State(ref db): State<Database>, Json(model): Json<institute::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let institute = institute::create(&db.conn, model).await?;
  Ok(Json(institute))
}

async fn get_institute(
  Extension(institute): Extension<institute::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(Json(institute))
}

async fn update_institute(
  State(ref db): State<Database>, Extension(institute): Extension<institute::Model>,
  Json(model): Json<institute::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let institute = institute::update(&db.conn, institute.id, model).await?;
  Ok(Json(institute))
}

async fn delete_institute(
  State(ref db): State<Database>, Extension(institute): Extension<institute::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  institute::delete(&db.conn, institute.id).await?;
  Ok(())
}
