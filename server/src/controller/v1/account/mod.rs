mod captcha;

use async_nats::jetstream;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{patch, post},
    Extension, Json, Router,
};
use nanoid::{alphabet, nanoid};
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{
    cache,
    cache::manager::RedisPool,
    captcha::captcha_protected,
    config::GlobalConfig,
    controller::{
        layer::{
            auth::{permission_required_all, Token, TokenTracker},
            info,
        },
        GlobalState,
    },
    email,
    email::{send_email, Email},
    entity::{
        config::Model as ConfigModel,
        user::{self, count_user, create_user, Permission, Permissions},
    },
    utility::hashing::{hash_password, verify_password},
};

pub fn router(state: &GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/send-verification-email", post(resend_verification_email))
        .route("/change-password", patch(change_password))
        .route("/self", patch(update_self_setting))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            info::prepare_user_full_info,
        ))
        .route("/delete", post(delete_self))
        .route("/logout", post(logout))
        .route_layer(middleware::from_fn(permission_required_all!(
            Permission::Basic
        )))
        .route("/reset-password", post(reset_password))
        .route("/send-reset-email", post(send_reset_email))
        .route("/verify-email", post(verify_email))
        .route("/login", post(login))
        .route("/register", post(register))
        .nest("/captcha", captcha::router(state))
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    pub account: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn login(
    State(ref db): State<DatabaseConnection>, State(ref mut cache): State<RedisPool>,
    Extension(config): Extension<ConfigModel>, Extension(token): Extension<Token>,
    Extension(token_tracker): Extension<TokenTracker>, Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("login request: {:?}", body);
    if token.id >= 0 {
        return Err((StatusCode::CONFLICT, "you are already logged in"));
    }
    let captcha_config = &config.captcha;
    captcha_protected!(
        captcha_config,
        cache,
        &body.captcha_id,
        &body.captcha_answer
    );

    let user = user::get_user_by_account(db, &body.account)
        .await
        .map_err(|err| {
            warn!("failed to get user by account: {:?}", err);
            match err {
                DbErr::RecordNotFound(_) => (StatusCode::FORBIDDEN, "account or password is wrong"),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to get user by account",
                ),
            }
        })?;

    if user.banned || !user.permissions.0.contains(&Permission::Basic) {
        return Err((StatusCode::FORBIDDEN, "account is banned"));
    }

    let password_hash = user.password.unwrap_or(String::new());

    match verify_password(&body.password, &password_hash) {
        Ok(true) => {
            *(token_tracker.token.lock().await) = Token {
                id: user.id,
                name: user.name,
                permissions: user.permissions,
                ..Default::default()
            };
            token_tracker
                .renew_requested
                .store(true, std::sync::atomic::Ordering::Relaxed);

            // NOTE: update user's password hash if it's not argon2
            if !password_hash.starts_with("$argon2") {
                if let Ok(password) = hash_password(&body.password).map_err(|err| {
                    warn!("failed to hash password: {:?}", err);
                }) {
                    user::update_user_password(db, user.id, password)
                        .await
                        .map_err(|err| {
                            error!("failed to update user password: {:?}", err);
                        })
                        .ok();
                }
            }

            Ok(StatusCode::OK)
        }
        Ok(false) => Err((StatusCode::FORBIDDEN, "account or password is wrong")),
        Err(err) => {
            warn!("failed to verify password: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to verify password with server error",
            ))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn register(
    State(ref db): State<DatabaseConnection>, State(ref mut cache): State<RedisPool>,
    State(ref queue): State<jetstream::Context>, State(global_config): State<GlobalConfig>,
    Extension(config): Extension<ConfigModel>, Extension(token_tracker): Extension<TokenTracker>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("register request: {:?}", body);
    let captcha_config = &config.captcha;
    captcha_protected!(
        captcha_config,
        cache,
        &body.captcha_id,
        &body.captcha_answer
    );

    if user::get_user_by_account(db, &body.email).await.is_ok() {
        return Err((StatusCode::CONFLICT, "account already exists"));
    }

    let password = hash_password(&body.password).map_err(|err| {
        warn!("failed to hash password: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to hash password")
    })?;

    let mut permissions = match count_user(db).await.map_err(|err| {
        error!("failed to count user: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "encountered error in database",
        )
    })? {
        0 => Permissions(vec![
            Permission::Basic,
            Permission::Verified,
            Permission::Publish,
            Permission::Audit,
            Permission::Organize,
            Permission::Devops,
            Permission::Statistics,
            Permission::Calendar,
            Permission::Certificates,
        ]),
        _ => Permissions(vec![Permission::Basic]),
    };
    if !config.email.enabled {
        permissions.0.push(Permission::Verified);
    }
    let email = body.email.clone();
    let new_user = user::Model {
        name: body.name,
        password: Some(password),
        email: Some(body.email),
        permissions,
        hidden: false,
        banned: false,
        ..Default::default()
    };

    let user = match create_user(db, new_user).await {
        Ok(user) => user,
        Err(err) => {
            error!("failed to create user: {:?}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create user with database error",
            ));
        }
    };
    if !config.email.enabled {
        return Ok(StatusCode::OK);
    }
    let verification_id = nanoid!(21, &alphabet::SAFE);
    match cache::Email::add_validation(cache, verification_id.as_str(), &email).await {
        Ok(_) => {}
        Err(err) => {
            error!("add email validation failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "add email validation failed",
            ));
        }
    }
    let email = Email {
        name: user.name.clone(),
        email: user.email.clone().unwrap_or_default(),
        subject: config.email.verify_email_subject.clone(),
        content: config
            .email
            .verify_email_body
            .replace(
                "%LINK%",
                &format!(
                    "{}/account/verify-email?email={}&token={}",
                    global_config.server.external_origin(),
                    user.email.unwrap_or_default(),
                    verification_id
                ),
            )
            .replace("%USER%", &user.name),
    };
    match send_email(&email, queue).await {
        Ok(_) => {
            *(token_tracker.token.lock().await) = Token {
                id: user.id,
                name: user.name,
                permissions: user.permissions,
                ..Default::default()
            };
            token_tracker
                .renew_requested
                .store(true, std::sync::atomic::Ordering::Relaxed);

            Ok(StatusCode::OK)
        }
        Err(err) => {
            error!("Failed to send verification email: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to send verification email",
            ))
        }
    }
}

async fn logout(
    State(ref mut cache): State<RedisPool>, Extension(token): Extension<Token>,
    Extension(token_tracker): Extension<TokenTracker>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    debug!("logout request");
    if token.id < 0 {
        return Err((StatusCode::FORBIDDEN, "you are not logged in"));
    }
    if let Some(token) = token_tracker.original {
        cache::Token::revoke(cache, &token).await.map_err(|err| {
            error!("failed to revoke token: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to revoke token with cache error",
            )
        })?;
    }

    Ok(StatusCode::OK)
}

async fn update_self_setting(
    State(ref conn): State<DatabaseConnection>, Extension(op_user): Extension<user::Model>,
    Json(data): Json<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    match user::update_self_profile(conn, op_user.id, data).await {
        Ok(user) => Ok(Json(user)),
        Err(err) => {
            error!("failed to update profile: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update user with database error",
            ))
        }
    }
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn change_password(
    State(ref conn): State<DatabaseConnection>, State(ref mut cache): State<RedisPool>,
    Extension(config): Extension<ConfigModel>, Extension(token_tracker): Extension<TokenTracker>,
    Extension(user): Extension<user::Model>, Json(body): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let captcha_config = &config.captcha;
    captcha_protected!(
        captcha_config,
        cache,
        &body.captcha_id,
        &body.captcha_answer
    );

    match verify_password(&body.old_password, &user.password.unwrap_or(String::new())) {
        Ok(true) => {}
        Ok(false) => return Err((StatusCode::FORBIDDEN, "account or password is wrong")),
        Err(err) => {
            warn!("failed to verify password: {:?}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to verify password with server error",
            ));
        }
    }
    let password = hash_password(&body.new_password).map_err(|err| {
        warn!("failed to hash password: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to hash password")
    })?;
    match user::update_user_password(conn, user.id, password).await {
        Ok(_) => {
            info!("Password changed for {}", user.name);
            if let Some(token) = token_tracker.original {
                cache::Token::revoke(cache, &token).await.map_err(|err| {
                    error!("failed to revoke token: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to revoke token with cache error",
                    )
                })?;
            }
            Ok(StatusCode::OK)
        }
        Err(DbErr::RecordNotFound(_)) => {
            error!("failed to update password: user {} not found", user.name);
            Err((StatusCode::NOT_FOUND, "user not found"))
        }
        Err(err) => {
            error!("failed to update password: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update password with database error",
            ))
        }
    }
}

#[derive(Deserialize)]
struct ResetEmailRequest {
    pub email: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn send_reset_email(
    State(ref mut cache): State<RedisPool>, State(ref conn): State<DatabaseConnection>,
    State(ref queue): State<jetstream::Context>, State(global_config): State<GlobalConfig>,
    Extension(config): Extension<ConfigModel>, Json(body): Json<ResetEmailRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let captcha_config = &config.captcha;
    captcha_protected!(
        captcha_config,
        cache,
        &body.captcha_id,
        &body.captcha_answer
    );

    match cache::Email::check_freq_limit(cache, &body.email).await {
        Ok(right) => {
            if !right {
                return Err((StatusCode::TOO_MANY_REQUESTS, "too many requests"));
            }
        }
        Err(err) => {
            error!("email check limit failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "email check limit failed",
            ));
        }
    }

    match user::get_user_by_account(conn, &body.email).await {
        Ok(user) => {
            if user.banned {
                return Err((StatusCode::FORBIDDEN, "account is banned"));
            }
        }
        Err(err) => {
            error!("failed to get user by email: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get user by email",
            ));
        }
    };
    let reset_id = nanoid!(21, &alphabet::SAFE);
    match cache::Email::add_validation(cache, reset_id.as_str(), &body.email).await {
        Ok(_) => {}
        Err(err) => {
            error!("add email validation failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "add email validation failed",
            ));
        }
    }
    let email = Email {
        name: body.email.clone(),
        email: body.email.clone(),
        subject: config.email.reset_password_email_subject.clone(),
        content: config.email.reset_password_email_body.replace(
            "%LINK%",
            &format!(
                "{}/account/reset-password?email={}&token={}",
                global_config.server.external_origin(),
                body.email,
                reset_id
            ),
        ),
    };
    match email::send_email(&email, queue).await {
        Ok(_) => {
            match cache::Email::add_freq_limit(cache, &body.email).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to add email limit: {}", err);
                }
            }
            Ok(StatusCode::OK)
        }
        Err(err) => {
            error!("Failed to send reset email: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to send reset email",
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResetPasswordRequest {
    pub email: String,
    pub password: String,
    pub token: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}

async fn reset_password(
    State(ref mut cache): State<RedisPool>, State(ref conn): State<DatabaseConnection>,
    Extension(config): Extension<ConfigModel>, Extension(token_tracker): Extension<TokenTracker>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let captcha_config = &config.captcha;
    captcha_protected!(
        captcha_config,
        cache,
        &body.captcha_id,
        &body.captcha_answer
    );

    let checked_email = match cache::Email::check_validation(cache, &body.token).await {
        Ok(Some(email)) => email,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "token not found"));
        }
        Err(err) => {
            error!("email check token failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "email check token failed",
            ));
        }
    };

    if checked_email != body.email {
        return Err((StatusCode::PRECONDITION_FAILED, "token not match email"));
    }

    let user = match user::get_user_by_account(conn, &body.email).await {
        Ok(user) => user,
        Err(err) => {
            error!("failed to get user by email: {:?}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get user by email",
            ));
        }
    };
    let password = hash_password(&body.password).map_err(|err| {
        warn!("failed to hash password: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to hash password")
    })?;
    match user::update_user_password(conn, user.id, password).await {
        Ok(_) => {
            info!("Password reset for {:?}", user.email);
            if let Some(token) = token_tracker.original {
                cache::Token::revoke(cache, &token).await.map_err(|err| {
                    error!("failed to revoke token: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to revoke token with cache error",
                    )
                })?;
            }
            Ok(StatusCode::OK)
        }
        Err(err) => {
            error!("Failed to update user password: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update user password",
            ))
        }
    }
}

async fn resend_verification_email(
    State(ref mut cache): State<RedisPool>, State(global_config): State<GlobalConfig>,
    State(ref queue): State<jetstream::Context>, Extension(config): Extension<ConfigModel>,
    Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    if user.permissions.0.contains(&Permission::Verified) {
        return Err((StatusCode::CONFLICT, "user already verified"));
    }

    match cache::Email::check_freq_limit(cache, &user.email.clone().unwrap_or_default()).await {
        Ok(right) => {
            if !right {
                return Err((StatusCode::TOO_MANY_REQUESTS, "too many requests"));
            }
        }
        Err(err) => {
            error!("email check limit failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "email check limit failed",
            ));
        }
    }

    // the email send in registration handler will not be record as limited,
    // user could resend the email immediately.
    match cache::Email::add_freq_limit(cache, &user.email.clone().unwrap_or_default()).await {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to add email limit: {}", err);
        }
    }
    let verification_id = nanoid!(21, &alphabet::SAFE);
    match cache::Email::add_validation(
        cache,
        &verification_id,
        &user.email.clone().unwrap_or_default(),
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to add email validation: {}", err);
        }
    }
    let email = Email {
        name: user.name.clone(),
        email: user.email.clone().unwrap_or_default(),
        subject: config.email.verify_email_subject.clone(),
        content: config
            .email
            .verify_email_body
            .replace(
                "%LINK%",
                &format!(
                    "{}/account/verify-email?email={}&token={}",
                    global_config.server.external_origin(),
                    user.email.unwrap_or_default(),
                    verification_id
                ),
            )
            .replace("%USER%", &user.name),
    };
    match send_email(&email, queue).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("Failed to send verification email: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to send verification email",
            ))
        }
    }
}

#[derive(Deserialize)]
struct VerifyEmailRequest {
    pub email: String,
    pub token: String,
}

async fn verify_email(
    State(ref mut cache): State<RedisPool>, State(ref conn): State<DatabaseConnection>,
    Extension(token_tracker): Extension<TokenTracker>, Json(body): Json<VerifyEmailRequest>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let checked_email = match cache::Email::check_validation(cache, &body.token).await {
        Ok(Some(email)) => email,
        Ok(None) => {
            return Err((StatusCode::NOT_FOUND, "token not found"));
        }
        Err(err) => {
            error!("email check token failed: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "email check token failed",
            ));
        }
    };

    if checked_email != body.email {
        return Err((StatusCode::BAD_REQUEST, "email not match"));
    }

    let mut user = match user::get_user_by_account(conn, &body.email).await {
        Ok(user) => user,
        Err(err) => {
            error!("failed to get user by email: {:?}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to get user by email",
            ));
        }
    };

    if user.permissions.0.contains(&Permission::Verified) {
        return Err((StatusCode::CONFLICT, "user already verified"));
    }
    user.permissions.0.push(Permission::Verified);

    match user::update_user(conn, user.id, user.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to update user: {:?}", err);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to update user"));
        }
    }

    // TODO: connect user and institute by email here.
    info!(
        "User verified: {} <{}>",
        user.name,
        user.email.unwrap_or_default()
    );
    if let Some(token) = token_tracker.original {
        cache::Token::revoke(cache, &token).await.map_err(|err| {
            error!("failed to revoke token: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to revoke token with cache error",
            )
        })?;
    }
    // renew token
    *(token_tracker.token.lock().await) = Token {
        id: user.id,
        name: user.name,
        permissions: user.permissions,
        ..Default::default()
    };
    token_tracker
        .renew_requested
        .store(true, std::sync::atomic::Ordering::Relaxed);

    Ok(StatusCode::OK)
}

async fn delete_self(
    State(ref conn): State<DatabaseConnection>, State(ref mut cache): State<RedisPool>,
    Extension(token): Extension<Token>,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let mut user = user::get_user(conn, token.id).await.map_err(|err| {
        error!("failed to get user: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to get user with database error",
        )
    })?;
    let user_count = user::count_activated_user(conn).await.map_err(|err| {
        error!("failed to count user: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to count user with database error",
        )
    })?;
    if user_count == 1 {
        return Err((
            StatusCode::FORBIDDEN,
            "you are the only one in this platform.",
        ));
    }
    user.name = format!("[DELETED]{}", user.id);
    user.email = Some(format!(
        "{}.deleted",
        user.email.unwrap_or("deleted@ret2shell".to_owned())
    ));
    user.institute_id = None;
    user.institute_info = None;
    user.intro = Some(String::new());
    user.permissions = Permissions::default();
    user.hidden = true;
    user.banned = true;
    user.cover_path = None;
    cache::Token::delete_all(cache, user.id)
        .await
        .map_err(|err| {
            error!("failed to delete all token: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to delete all token with cache error",
            )
        })?;
    match user::update_user(conn, user.id, user).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            error!("failed to update user: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to update user with database error",
            ))
        }
    }
}
