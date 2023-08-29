//! Generating captcha and verifying the captcha.
//!
//!

pub mod hcaptcha;
pub mod image;
pub mod pow;
pub mod recaptcha;
mod traits;

pub use traits::{Captcha, Validator};

use self::traits::{CaptchaError, CaptchaValidator};
use crate::cache::manager::RedisPool;

pub async fn generate_captcha(
    validator: Validator,
    conn: &mut RedisPool,
    difficulty: u16,
) -> Result<Captcha, CaptchaError> {
    match validator {
        Validator::None => Ok(Captcha {
            id: "".to_string(),
            validator: Validator::None,
            challenge: "".to_string(),
            answer: "".to_string(),
        }),
        Validator::Image => Ok(image::ImageValidator::generate_captcha(conn, difficulty).await?),
        Validator::Pow => Ok(pow::PowValidator::generate_captcha(conn, difficulty).await?),
        Validator::RecaptchaV3 => {
            Ok(recaptcha::ReCaptchaV3Validator::generate_captcha(conn, difficulty).await?)
        }
        Validator::HCaptcha => {
            Ok(hcaptcha::HCaptchaValidator::generate_captcha(conn, difficulty).await?)
        }
    }
}

pub async fn check_captcha(
    validator: Validator,
    conn: &mut RedisPool,
    difficulty: u16,
    id: &str,
    answer: &str,
) -> Result<bool, CaptchaError> {
    match validator {
        Validator::None => Ok(true),
        Validator::Image => {
            image::ImageValidator::check_captcha(conn, difficulty, id, answer).await
        }
        Validator::Pow => pow::PowValidator::check_captcha(conn, difficulty, id, answer).await,
        Validator::RecaptchaV3 => {
            recaptcha::ReCaptchaV3Validator::check_captcha(conn, difficulty, id, answer).await
        }
        Validator::HCaptcha => {
            hcaptcha::HCaptchaValidator::check_captcha(conn, difficulty, id, answer).await
        }
    }
}
