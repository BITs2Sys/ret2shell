use axum::{Router, middleware, routing::get};
use r2s_database::user::Permission;

use crate::{middleware::auth, traits::GlobalState};

mod direct;
mod serve;
mod source;

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .nest("/v1", serve::router(state))
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
        )
        .route_layer(middleware::from_fn(auth::permission_required_all!(
          Permission::Basic,
          Permission::Verified,
          Permission::DevOps
        ))),
    )
    .nest(
      "/direct",
      Router::new()
        .route(
          "/discover",
          axum::routing::post(direct::discover_remote_source),
        )
        .route_layer(middleware::from_fn(auth::permission_required_any!(
          Permission::Host,
          Permission::DevOps
        )))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
          Permission::Basic,
          Permission::Verified
        ))),
    )
}
