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

    /// Paths of the file(s)/directory(ies) to encrypt
    #[clap(value_parser)]
    paths: Vec<String>,

    /// Set output path (only supported when one input path
    /// provided)
    #[clap(short, long, value_parser)]
    output_path: Option<String>,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(args.key.as_bytes());

    // Runs for every provided input path
    for input_path in &args.paths {
        let input_path = PathBuf::from(input_path);
        let output_path = match &args.output_path {
            Some(v) => {
                if &args.paths.len() == &1usize {
                    PathBuf::from(v)
                } else {
                    panic!("Only one input path can be provided when setting an output path");
                }
            }
            None => {
                let mut path = PathBuf::from(&input_path);
                path.set_extension("enc");
                path
            }
        };

        // Reads entry metadata to act in consequence
        let entry_metadata = fs::metadata(&input_path).expect("Failed to read entry metadata");
        if entry_metadata.file_type().is_dir() {
            // The case where the entry is a directory

            // Creates base directory to put encrypted files
            // in
            if let Err(e) = fs::create_dir(&output_path) {
                match e.kind() {
                    io::ErrorKind::AlreadyExists => (),
                    e => panic!("Failed to create directory: {}", e),
                };
            };

            let walk_dir = WalkDir::new(&input_path).expect("Failed to read directory");

            // Runs for every entry in the specified directory
            for entry in walk_dir {
                let entry = entry.expect("Failed to read entry");
                let entry_path = entry.path();
                let new_entry_path = output_path.join(
                    entry_path
                        .strip_prefix(&input_path)
                        .expect("Failed to establish relative file path"),
                );

                let entry_type = entry.file_type().expect("Failed to read file type");
                if entry_type.is_dir() {
                    if let Err(e) = fs::create_dir(&new_entry_path) {
                        match e.kind() {
                            io::ErrorKind::AlreadyExists => (),
                            e => panic!("Failed to create directory: {}", e),
                        };
                    };
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
                };
            }
        } else if entry_metadata.file_type().is_file() {
            // The case where the entry is a file

            print!("{} ... ", input_path.display());

            let mut source = File::open(&input_path).expect("Failed to read source file");
            let mut dest = File::create(&output_path).expect("Failed to create destination file");

            match crypto.encrypt_stream(&mut source, &mut dest) {
                Ok(_) => println!("Ok"),
                Err(e) => panic!("Failed to encrypt: {}", e),
            };
        } else {
            // The case where the entry is something else
            println!("Skipped entry: Unknown type");
        }
    }

    println!(
        "Done: All Ok ({}ms elapsed)",
        timer.elapsed().unwrap_or_default().as_millis()
    )
}
