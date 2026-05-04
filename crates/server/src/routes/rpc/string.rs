use axum::{Router, extract::Query, response::IntoResponse, routing::get};
use serde::Deserialize;

use crate::{
  traits::{GlobalState, ResponseError},
  utility::string::{deunicode_str, leet_str},
};

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .route("/deunicode", get(generate_deunicode))
    .route("/leet", get(generate_leet))
}

#[derive(Deserialize)]
struct GenericQuery {
  text: String,
  keep_case: Option<bool>,
}

async fn generate_deunicode(
  Query(query): Query<GenericQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(deunicode_str(query.text, query.keep_case.unwrap_or(false)))
}

async fn generate_leet(
  Query(query): Query<GenericQuery>,
) -> Result<impl IntoResponse, ResponseError> {
  Ok(leet_str(query.text))
}
