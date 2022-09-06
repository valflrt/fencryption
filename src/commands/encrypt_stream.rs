use std::fs::File;

use clap::Args;

use crate::crypto::Crypto;

#[derive(Args)]
/// Encrypts text using the passed key
pub struct Command {
    /// Key used to encrypt
    #[clap(value_parser)]
    key: String,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let crypto = Crypto::new(args.key.as_bytes());

    let mut source = File::open("_test.bin").expect("Failed to read source file");
    let mut dest = File::create("_test.enc.bin").expect("Failed to create destination file");

    match crypto.encrypt_stream(&mut source, &mut dest) {
        Ok(_) => println!("Success"),
        Err(e) => panic!("Failed to encrypt: {}", e),
    };
}
