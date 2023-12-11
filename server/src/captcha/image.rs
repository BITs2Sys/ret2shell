use async_trait::async_trait;

use crate::{
    cache::{self, manager::RedisPool},
    captcha::traits::{Captcha, CaptchaError, CaptchaValidator, Validator},
};

pub struct ImageValidator;

#[async_trait]
impl CaptchaValidator for ImageValidator {
    async fn generate_captcha(
        conn: &mut RedisPool, difficulty: u16,
    ) -> Result<Captcha, CaptchaError> {
        let (answer, challenge) = biosvg::BiosvgBuilder::new()
            .length(4)
            .difficulty(difficulty)
            .colors(vec![
                "#0078D6".to_string(),
                "#aa3333".to_string(),
                "#f08012".to_string(),
                "#33aa00".to_string(),
                "#AA00AA".to_string(),
                "#44CC7F".to_string(),
            ])
            .build()
            .map_err(|_| CaptchaError::BuilderError)?;
        let id = nanoid::nanoid!();

        cache::Captcha::store(conn, &id, &answer).await?;

        Ok(Captcha {
            id,
            validator: Validator::Image,
            challenge,
            answer,
        })
    }

    async fn check_captcha(
        conn: &mut RedisPool, _difficulty: u16, id: &str, answer: &str,
    ) -> Result<bool, CaptchaError> {
        let criteria = cache::Captcha::get(conn, id).await?;
        Ok(criteria.trim().to_lowercase() == answer.trim().to_lowercase())
    }
}
