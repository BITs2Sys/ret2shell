use async_trait::async_trait;

use super::traits::{Captcha, CaptchaError, CaptchaValidator, Validator};
use crate::{
    cache::{self, manager::RedisPool},
    utility::hashing::sha256sum_str,
};
pub struct PowValidator;

#[async_trait]
impl CaptchaValidator for PowValidator {
    async fn generate_captcha(
        conn: &mut RedisPool, difficulty: u16,
    ) -> Result<Captcha, CaptchaError> {
        let id = nanoid::nanoid!();
        let challenge = nanoid::nanoid!(16);
        cache::Captcha::store(conn, &id, &challenge).await?;
        Ok(Captcha {
            id,
            validator: Validator::Pow,
            challenge: format!("{}#{}", difficulty, challenge),
            answer: "".to_string(),
        })
    }

    async fn check_captcha(
        conn: &mut RedisPool, difficulty: u16, id: &str, answer: &str,
    ) -> Result<bool, CaptchaError> {
        let criteria = cache::Captcha::get(conn, id).await?;
        if answer.trim().starts_with(criteria.trim())
            && sha256sum_str(answer.trim()).starts_with("0".repeat(difficulty as usize).as_str())
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
