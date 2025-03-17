use axum::{Router, extract::Query, response::IntoResponse, routing::get};
use deunicode::deunicode_with_tofu;
use heck::ToSnakeCase;
use serde::Deserialize;

use crate::{
  traits::{GlobalState, ResponseError},
  utility::string::leet_str,
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/deunicode", get(generate_deunicode))
    .route("/leet", get(generate_leet))
}

#[derive(Deserialize)]
struct GenericQuery {
  text: String,
}

async fn generate_deunicode(
  Query(query): Query<GenericQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(
    deunicode_with_tofu(&query.text, "_")
      .trim()
      .to_owned()
      .to_snake_case(),
  )
}

async fn generate_leet(
  Query(query): Query<GenericQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(leet_str(query.text))
}
