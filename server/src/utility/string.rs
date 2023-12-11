//! String utils
//!
//! Currently contains deunicode function.

use deunicode::deunicode_with_tofu;
use sanitizer::StringSanitizer;

/// Convert any unicode string into a path-safe string.
///
/// Assume that we have a string "你好世界", it will be converted to
/// "ni_hao_shi_jie".
///
/// It maybe slow due to a huge map lookup, so plz do not use it with a long
/// input.
#[allow(dead_code)]
pub fn deunicode_str(s: impl AsRef<str>) -> String {
    let mut sanitizer = StringSanitizer::from(deunicode_with_tofu(s.as_ref(), "_"));
    sanitizer.trim().to_snake_case();
    sanitizer.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deunicode_str() {
        assert_eq!(deunicode_str("你好世界@#$^%^%*&#"), "ni_hao_shi_jie");
        assert_eq!(
            deunicode_str("✨1日目✨#原神アプデ記念キャンペーン！"),
            "sparkles_1ri_mu_sparkles_yuan_shen_apude_ji_nian_kiyanpen"
        );
    }
}
