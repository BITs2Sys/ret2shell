use axum::{
  extract::{Query, State},
  middleware,
  response::IntoResponse,
  routing::get,
  Extension, Json, Router,
};
use r2s_database::user::{self, Permission};
use r2s_migrator::Database;
use serde::Deserialize;

use crate::{
  middleware::{auth::Token, data},
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .nest(
      "/:user",
      Router::new()
        .route("/", get(get_user))
        .route_layer(middleware::from_fn_with_state(
          state.clone(),
          data::prepare_data!(user, false),
        )),
    )
    .route("/", get(get_user_list))
}

#[derive(Deserialize)]
struct UserListQuery {
  page: Option<u64>,
  page_size: Option<u64>,
  order: Option<String>,
  filter: Option<String>,
  with_institute_id: Option<i64>,
}

async fn get_user_list(
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Query(query): Query<UserListQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  let results = user::get_page(
    &db.conn,
    query.page.unwrap_or(1),
    query.page_size.unwrap_or(15),
    query.order,
    query.filter,
    token.permissions.0.contains(&user::Permission::User),
    query.with_institute_id,
  )
  .await?;
  if token.permissions.0.contains(&Permission::User) {
    Ok(Json(results))
  } else {
    Ok(Json((
      results.0.into_iter().map(|r| r.desensitize()).collect(),
      results.1,
    )))
  }
}

async fn get_user(
  Extension(user): Extension<user::Model>, Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, ResponseError> {
  if token.permissions.0.contains(&Permission::User) {
    Ok(Json(user))
  } else {
    Ok(Json(user.desensitize()))
  }
}
