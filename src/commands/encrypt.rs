use base64;
use clap::Args;

use crate::crypto::Crypto;

#[derive(Args)]
/// Encrypts text using the passed key
pub struct Command {
    /// Key used to encrypt
    #[clap(value_parser)]
    key: String,

    /// Data to encrypt
    #[clap(value_parser)]
    plain_data: String,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let crypto = Crypto::new(args.key.as_bytes());

    let enc_data = match crypto.encrypt(args.plain_data.as_bytes()) {
        Ok(enc) => enc,
        Err(e) => panic!("Failed to encrypt: {}", e),
    };

    println!("\nEncryption result:\n");
    println!("- byte array result: {:x?}", &enc_data);
    println!("- base 64 encoded result: {}", base64::encode(&enc_data));
}
