use axum::{
  Extension, Json, Router,
  extract::{Query, State},
  middleware,
  response::IntoResponse,
  routing::{get, patch},
};
use r2s_cache::Cache;
use r2s_database::{
  ip, oauth, team,
  user::{self, Permission},
};
use r2s_migrator::Database;
use serde::Deserialize;

use crate::{
  middleware::{
    auth::{self, Token, TokenTracker},
    data,
  },
  traits::{GlobalState, ResponseError},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
  Router::new()
    .nest(
      "/{user}",
      Router::new()
        .route("/oauth", get(get_oauth_list))
        .route("/ip", get(get_user_ip_list))
        .route("/", patch(update_user).delete(delete_user))
        .route_layer(middleware::from_fn(auth::permission_required_all!(
          Permission::User
        )))
        .route("/", get(get_user))
        .route("/team", get(get_teams))
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
  institute_id: Option<i64>,
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
    query.institute_id,
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
  State(ref db): State<Database>, Extension(token): Extension<Token>,
  Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let user = user::get_ex(&db.conn, user.id).await?;
  let user = user.ok_or_else(|| ResponseError::NotFound("user not found".to_owned()))?;
  if token.permissions.0.contains(&Permission::User) || user.id == token.id {
    Ok(Json(user))
  } else if !user.hidden {
    Ok(Json(user.desensitize()))
  } else {
    Err(ResponseError::NotFound("user not found".to_owned()))
  }
}

async fn get_teams(
  State(ref db): State<Database>, Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let teams = team::get_list_by_user_id_ex(&db.conn, user.id).await?;
  Ok(Json(
    teams
      .into_iter()
      .map(|t| t.desensitize())
      .collect::<Vec<_>>(),
  ))
}

async fn logout_user(cache: &Cache, user_id: i64) -> Result<(), ResponseError> {
  while let Some(token) = cache
    .at("token")
    .pop::<String>(format!("user-{user_id}"))
    .await?
  {
    cache.at("token").del(&token).await.ok();
  }
  cache.del(format!("user-{user_id}")).await.ok();
  Ok(())
}

async fn update_user(
  State(ref db): State<Database>, State(ref cache): State<Cache>,
  Extension(user): Extension<user::Model>, Extension(token): Extension<Token>,
  Extension(token_tracker): Extension<TokenTracker>, Json(data): Json<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let user = user::update(
    &db.conn,
    user::Model {
      account: data.account,
      nickname: data.nickname,
      email: data.email,
      description: data.description,
      avatar: data.avatar,
      institute_id: data.institute_id,
      permissions: data.permissions,
      hidden: data.hidden,
      banned: data.banned,
      ..user
    },
  )
  .await?;

  logout_user(cache, user.id).await?;

  if token.id == user.id {
    *(token_tracker.token.lock().await) = Token {
      id: user.id.clone(),
      account: user.account.clone(),
      nickname: user.nickname.clone(),
      permissions: user.permissions.clone(),
      ..Default::default()
    };
    token_tracker
      .renew_requested
      .store(true, std::sync::atomic::Ordering::Relaxed);
  }

  Ok(Json(user))
}

async fn delete_user(
  State(ref db): State<Database>, State(ref cache): State<Cache>,
  Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  user::delete(&db.conn, user.id).await?;
  logout_user(cache, user.id).await?;
  Ok(Json(user))
}

async fn get_user_ip_list(
  State(ref db): State<Database>, Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let ips = ip::get_list(&db.conn, user.id).await?;
  Ok(Json(ips))
}

async fn get_oauth_list(
  State(ref db): State<Database>, Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, ResponseError> {
  let oauths = oauth::get_list_ex(&db.conn, user.id).await?;
  Ok(Json(oauths))
}
