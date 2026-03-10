use axum::{Router, middleware, routing::get};
use r2s_database::user::Permission;

use crate::{middleware::auth, traits::GlobalState};

mod source;

pub fn router(_state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .nest(
      "/source",
      Router::new()
        .route(
          "/",
          get(source::list_registry_sources).post(source::create_registry_source),
        )
        .route(
          "/{source}",
          axum::routing::patch(source::update_registry_source)
            .delete(source::delete_registry_source),
        )
        .route(
          "/{source}/fetch",
          axum::routing::post(source::fetch_registry_source),
        ),
    )
    .route_layer(middleware::from_fn(auth::permission_required_all!(
      Permission::Basic,
      Permission::Verified,
      Permission::DevOps
    )))
}
