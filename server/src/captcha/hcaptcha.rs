use async_trait::async_trait;

use crate::cache::manager::RedisPool;

use super::traits::{Captcha, CaptchaError, CaptchaValidator};
pub struct HCaptchaValidator;

#[async_trait]
impl CaptchaValidator for HCaptchaValidator {
    async fn generate_captcha(
        _conn: &mut RedisPool,
        _difficulty: u16,
    ) -> Result<Captcha, CaptchaError> {
        Err(CaptchaError::Unknown)
    }

    async fn check_captcha(
        _conn: &mut RedisPool,
        _difficulty: u16,
        _id: &str,
        _answer: &str,
    ) -> Result<bool, CaptchaError> {
        Err(CaptchaError::Unknown)
    }
}
