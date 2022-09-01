use base64;
use clap::Args as ClapArgs;

use fencryption_rust::crypto::Crypto;

#[derive(ClapArgs)]
pub struct Args {
    /// key used to encrypt
    #[clap(value_parser)]
    key: String,

    /// Data to encrypt
    #[clap(value_parser)]
    plain_data: String,
}

pub fn action(args: &Args) {
    let crypto = Crypto::new(args.key.as_bytes().to_vec());

    let encrypted = match crypto.encrypt(args.plain_data.as_bytes()) {
        Ok(enc) => enc,
        Err(e) => panic!("Failed to encrypt: {}", e),
    };

    println!("byte array result: {:x?}", &encrypted);
    println!("base 64 encoded result: {}", base64::encode(&encrypted));
}
