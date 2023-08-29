use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use sea_orm::DatabaseConnection;

use crate::cache::{manager::RedisPool, platform::Platform};

pub async fn prepare_platform_info<B>(
    State(ref db): State<DatabaseConnection>,
    State(ref mut pool): State<RedisPool>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let platform_info = Platform::get(pool, db).await.map_err(|err| {
        tracing::error!("query platform info from cache failed: {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "encountered cache error")
    })?;
    if let Some(platform_info) = platform_info.clone() {
        req.extensions_mut().insert(platform_info);
    }
    Ok(next.run(req).await)
}
