use std::fs::File;

use clap::Args;

use crate::crypto::Crypto;

#[derive(Args)]
/// Encrypts text using the passed key
pub struct Command {
    /// Key used to encrypt
    #[clap(value_parser)]
    key: String,

    /// Path of the file to encrypt
    #[clap(value_parser)]
    path: String,

    /// Output path where to write the encrypted data
    #[clap(short, long, value_parser)]
    output_path: Option<String>,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let crypto = Crypto::new(args.key.as_bytes());

    let output_path;
    match &args.output_path {
        Some(v) => output_path = v.to_string(),
        None => output_path = [&args.path, ".enc"].join(""),
    };

    let mut source = File::open(&args.path).expect("Failed to read source file");
    let mut dest = File::create(&output_path).expect("Failed to create destination file");

    match crypto.encrypt_stream(&mut source, &mut dest) {
        Ok(_) => println!("Success"),
        Err(e) => panic!("Failed to encrypt: {}", e),
    };
}
