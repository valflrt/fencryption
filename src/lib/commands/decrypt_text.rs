use base64::{engine::general_purpose, Engine};

use crate::{
    commands::{ErrorBuilder, Result},
    crypto::Crypto,
};

pub fn execute(key: &String, encrypted: &String) -> Result<String> {
    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let enc = general_purpose::STANDARD
        .decode(encrypted.to_owned())
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to decode base64")
                .error(e)
                .build()
        })?;

    let dec = crypto.decrypt(enc).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to encrypt text")
            .error(e)
            .build()
    })?;

    Ok(String::from_utf8(dec).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to decode decrypted bytes")
            .error(e)
            .build()
    })?)
}
