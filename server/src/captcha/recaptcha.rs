use async_trait::async_trait;

use super::traits::{Captcha, CaptchaError, CaptchaValidator};
use crate::cache::manager::RedisPool;
pub struct ReCaptchaV3Validator;

#[async_trait]
impl CaptchaValidator for ReCaptchaV3Validator {
    async fn generate_captcha(
        _conn: &mut RedisPool, _difficulty: u16,
    ) -> Result<Captcha, CaptchaError> {
        Err(CaptchaError::Unknown)
    }

    async fn check_captcha(
        _conn: &mut RedisPool, _difficulty: u16, _id: &str, _answer: &str,
    ) -> Result<bool, CaptchaError> {
        Err(CaptchaError::Unknown)
    }
}
