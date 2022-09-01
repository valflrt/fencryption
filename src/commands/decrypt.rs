use base64;
use clap::Args as ClapArgs;

use fencryption_rust::crypto::Crypto;

#[derive(ClapArgs)]
pub struct Args {
    /// key used to decrypt
    #[clap(value_parser)]
    key: String,

    /// Data to decrypt
    #[clap(value_parser)]
    encrypted_data: String,
}

pub fn action(args: &Args) {
    let crypto = Crypto::new(args.key.as_bytes().to_vec());

    let encrypted_data =
        base64::decode(args.encrypted_data.as_bytes()).expect("Wrongly base64 encoded data");

    let decrypted = match crypto.decrypt(&encrypted_data) {
        Ok(dec) => dec,
        Err(e) => panic!("Failed to decrypt: {}", e),
    };

    println!("byte array result: {:x?}", &decrypted);
    println!("base 64 encoded result: {}", base64::encode(&decrypted));
    println!(
        "utf8 encoded result: {}",
        match std::str::from_utf8(&decrypted) {
            Ok(v) => v,
            Err(_) => "Invalid utf8 sequence",
        }
    );
}
