use base64;
use clap::Args as ClapArgs;

use fencryption_rust::crypto::Crypto;

#[derive(ClapArgs)]
/// Encrypts text using the passed key
pub struct Command {
    /// Key used to encrypt
    #[clap(value_parser)]
    key: String,

    /// Data to encrypt
    #[clap(value_parser)]
    plain_data: String,
}

pub fn action(args: &Command) {
    let crypto = Crypto::new(args.key.as_bytes().to_vec());

    let enc_data = match crypto.encrypt(args.plain_data.as_bytes()) {
        Ok(enc) => enc,
        Err(e) => panic!("Failed to encrypt: {}", e),
    };

    println!("\nEncryption result:\n");
    println!("- byte array result: {:x?}", &enc_data);
    println!("- base 64 encoded result: {}", base64::encode(&enc_data));
}
