use base64::{engine::general_purpose, Engine};
use clap::{arg, Args};
use fencryption_lib::crypto::Crypto;

use crate::{error::ErrorBuilder, log, logic, result::Result};

#[derive(Args, Clone)]
/// Decrypt text
pub struct Command {
    /// Text to decrypt (in base64)
    #[arg(required = true)]
    encrypted: String,
}

pub fn execute(args: &Command) -> Result<()> {
    let key = logic::prompt_key(false)?;

    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let enc = general_purpose::STANDARD
        .decode(args.encrypted.to_owned())
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

    log::println_success("Successfully decrypted text:");

    println!(
        "{}",
        String::from_utf8(dec).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to decode decrypted bytes")
                .error(e)
                .build()
        })?
    );

    Ok(())
}
