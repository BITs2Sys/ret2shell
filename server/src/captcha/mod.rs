//! Generating captcha and verifying the captcha.

pub mod hcaptcha;
pub mod image;
pub mod pow;
pub mod recaptcha;
mod traits;

pub use traits::{Captcha, Validator};

pub use self::traits::{CaptchaError, CaptchaValidator};
use crate::cache::manager::RedisPool;

pub async fn generate_captcha(
    validator: &Validator, conn: &mut RedisPool, difficulty: &u16,
) -> Result<Captcha, CaptchaError> {
    match validator {
        Validator::None => Ok(Captcha {
            id: "".to_string(),
            validator: Validator::None,
            challenge: "".to_string(),
            answer: "".to_string(),
        }),
        Validator::Image => Ok(image::ImageValidator::generate_captcha(conn, *difficulty).await?),
        Validator::Pow => Ok(pow::PowValidator::generate_captcha(conn, *difficulty).await?),
        Validator::RecaptchaV3 => {
            Ok(recaptcha::ReCaptchaV3Validator::generate_captcha(conn, *difficulty).await?)
        }
        Validator::HCaptcha => {
            Ok(hcaptcha::HCaptchaValidator::generate_captcha(conn, *difficulty).await?)
        }
    }
}

pub async fn check_captcha(
    validator: &Validator, conn: &mut RedisPool, difficulty: &u16, id: &str, answer: &str,
) -> Result<bool, CaptchaError> {
    match validator {
        Validator::None => Ok(true),
        Validator::Image => {
            image::ImageValidator::check_captcha(conn, *difficulty, id, answer).await
        }
        Validator::Pow => pow::PowValidator::check_captcha(conn, *difficulty, id, answer).await,
        Validator::RecaptchaV3 => {
            recaptcha::ReCaptchaV3Validator::check_captcha(conn, *difficulty, id, answer).await
        }
        Validator::HCaptcha => {
            hcaptcha::HCaptchaValidator::check_captcha(conn, *difficulty, id, answer).await
        }
    }
}

macro_rules! captcha_protected {
    ($cache_config:expr, $cache:expr, $id:expr, $answer:expr) => {
        if $cache_config.enabled {
            match crate::captcha::check_captcha(
                &$cache_config.validator,
                $cache,
                &$cache_config.difficulty,
                $id,
                $answer,
            )
            .await
            {
                Ok(true) => {}
                Ok(false) => {
                    return Err((axum::http::StatusCode::UNAUTHORIZED, "hey robot"));
                }
                Err(crate::captcha::CaptchaError::Unknown) => {
                    tracing::error!("captcha check failed with unknown reason");
                    return Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "captcha check failed with unknown reason",
                    ));
                }
                Err(crate::captcha::CaptchaError::CacheError(err)) => {
                    tracing::warn!("encountered captcha cache error: {:?}", err);
                    return Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "captcha is outdate",
                    ));
                }
                Err(crate::captcha::CaptchaError::BuilderError) => {
                    tracing::error!("failed to build captcha validator");
                    return Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to build captcha validator",
                    ));
                }
            }
        };
    };
}

pub(crate) use captcha_protected;
