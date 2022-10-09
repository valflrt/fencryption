use std::{
    fs::{self, File},
    io,
    path::PathBuf,
    time,
};

use clap::Args;

use fencryption::{crypto::Crypto, walk_dir::WalkDir};

#[derive(Args)]
/// Encrypt specified file/directory using the passed key
pub struct Command {
    /// Key used to encrypt
    #[clap(value_parser)]
    key: String,

    /// Path of the file/directory to encrypt
    #[clap(value_parser)]
    path: String,

    /// Set output path
    #[clap(short, long, value_parser)]
    output_path: Option<String>,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(args.key.as_bytes());

    let input_dir_path = PathBuf::from(&args.path);
    let output_dir_path = match &args.output_path {
        Some(v) => PathBuf::from(v),
        None => {
            let mut path = PathBuf::from(&args.path);
            path.set_extension("enc");
            path
        }
    };

    match fs::create_dir(&output_dir_path) {
        Ok(_) => println!("created base directory: \"{}\"", &output_dir_path.display()),
        Err(e) => match e.kind() {
            io::ErrorKind::AlreadyExists => (),
            e => panic!("Failed to create directory: {}", e),
        },
    }

    let walk_dir = WalkDir::new(&input_dir_path).expect("Error: Failed to read directory.");

    for entry in walk_dir {
        let entry = entry.expect("Failed to read entry");

        let entry_path = entry.path();

        let new_entry_path = output_dir_path.join(
            entry_path
                .strip_prefix(&input_dir_path)
                .expect("Failed to establish relative file path"),
        );

        let entry_type = entry.file_type().expect("Failed to read file type");

        if entry_type.is_dir() {
            match fs::create_dir(&new_entry_path) {
                Ok(_) => println!("Created sub-directory: \"{}\"", &new_entry_path.display()),
                Err(e) => match e.kind() {
                    io::ErrorKind::AlreadyExists => (),
                    e => panic!("Failed to create directory: {}", e),
                },
            }
        } else if entry_type.is_file() {
            print!("{} ... ", &entry_path.display());

            let mut source = File::open(&entry_path).expect("Failed to read source file");
            let mut dest =
                File::create(&new_entry_path).expect("Failed to create destination file");

            match crypto.encrypt_stream(&mut source, &mut dest) {
                Ok(_) => println!("Ok"),
                Err(e) => panic!("Failed to encrypt: {}", e),
            };
        } else {
            println!("Skipped entry: Unknown type");
        }
    }

    println!(
        "Done: All Ok ({}ms elapsed)",
        timer.elapsed().unwrap_or_default().as_millis()
    )
}
