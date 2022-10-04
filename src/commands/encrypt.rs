use std::{fs::File, path::PathBuf};

use clap::Args;

use crate::{crypto::Crypto, walk_dir::WalkDir};

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

    let input_dir_path = PathBuf::from(&args.path);

    let output_dir_path = match &args.output_path {
        Some(v) => PathBuf::from(v),
        None => PathBuf::from(&args.path).join(".enc"),
    };

    let walk_dir = WalkDir::new(&input_dir_path).expect("Error: Failed to read directory.");

    for entry in walk_dir {
        match entry {
            Ok(entry) => {
                let entry_path = entry.path();
                let relative_entry_path = match entry_path.strip_prefix(&args.path) {
                    Ok(v) => output_dir_path.join(v),
                    Err(e) => panic!("Failed to establish relative file path, {}", e),
                };

                println!("{}", &relative_entry_path.display());

                let mut source = File::open(&args.path).expect("Failed to read source file");
                let mut dest =
                    File::create(relative_entry_path).expect("Failed to create destination file");

                match crypto.encrypt_stream(&mut source, &mut dest) {
                    Ok(_) => println!("Success"),
                    Err(e) => panic!("Failed to encrypt: {}", e),
                };
            }
            Err(e) => panic!("{:#?}", e),
        }
    }
}
