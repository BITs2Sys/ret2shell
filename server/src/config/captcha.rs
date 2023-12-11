use serde::{Deserialize, Serialize};

use crate::captcha::Validator;

/// Captcha configuration struct.
///
/// This struct contains the configuration settings for the captcha system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaConfig {
    /// Whether the captcha system is enabled or not.
    pub enabled: bool,
    /// The time in seconds before the captcha expires.
    pub expires_time: i64,
    /// The validator used for the captcha.
    pub validator: Validator,
    /// The type of captcha used: 'image' or 'pow'.
    pub difficulty: u16, // 1-10
}
