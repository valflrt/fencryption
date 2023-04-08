use base64::{engine::general_purpose, Engine};
use clap::{arg, Args};
use fencryption_lib::crypto::Crypto;

use crate::{error::ErrorBuilder, log, logic, result::Result};

#[derive(Args, Clone)]
/// Encrypt text
pub struct Command {
    /// Text to encrypt
    #[arg(required = true)]
    text: String,
}

pub fn execute(args: &Command) -> Result<()> {
    let key = logic::prompt_key(true)?;

    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let plain = args.text.as_bytes();

    let enc = crypto.encrypt(plain).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to encrypt text")
            .error(e)
            .build()
    })?;

    log::println_success("Successfully encrypted text:");

    println!("base64 {}", general_purpose::STANDARD.encode(enc));

    Ok(())
}
