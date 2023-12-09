//! Hashing utility functions

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use ring::digest::{Context, SHA256};
use thiserror::Error;

/// Calculate the sha256 value of a string, returns a hex string.
pub fn sha256sum_str(message: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(message.as_bytes());
    hex::encode(context.finish().as_ref())
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

#[derive(Error, Debug)]
pub enum PasswordHashingError {
    #[error("bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("argon2 error: {0}")]
    Argon2Error(argon2::password_hash::Error),
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordHashingError> {
    if hash.starts_with("$argon2") {
        let parsed_hash = PasswordHash::new(hash).map_err(PasswordHashingError::Argon2Error)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    } else {
        Ok(bcrypt::verify(password, hash)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::utility::hashing::sha256sum_str;

    #[test]
    fn test_sha256() {
        assert!(sha256sum_str("Hello World!")
            .eq("7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"))
    }
}
