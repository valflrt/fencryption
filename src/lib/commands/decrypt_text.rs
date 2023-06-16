//! Decrypt text.

use base64::{engine::general_purpose, Engine};

use crate::{
    commands::{ErrorBuilder, Result},
    crypto::Crypto,
};

/// Decrypts given text (from base64 encoded text).
pub fn execute(key: &String, encrypted: &String) -> Result<String> {
    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::default()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let enc = general_purpose::STANDARD.decode(encrypted).map_err(|e| {
        ErrorBuilder::default()
            .message("Failed to decode base64")
            .error(e)
            .build()
    })?;

    let dec = crypto.decrypt(enc).map_err(|e| {
        ErrorBuilder::default()
            .message("Failed to encrypt text")
            .error(e)
            .build()
    })?;

    String::from_utf8(dec).map_err(|e| {
        ErrorBuilder::default()
            .message("Failed to decode decrypted bytes")
            .error(e)
            .build()
    })
}
