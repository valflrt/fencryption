//! Encrypt text.

use base64::{engine::general_purpose, Engine};

use crate::{
    commands::{ErrorBuilder, Result},
    crypto::Crypto,
};

/// Encrypts given text.
pub fn execute(key: &String, text: &String) -> Result<String> {
    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let plain = text.as_bytes();

    let enc = crypto.encrypt(plain).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to encrypt text")
            .error(e)
            .build()
    })?;

    Ok(general_purpose::STANDARD.encode(enc))
}
