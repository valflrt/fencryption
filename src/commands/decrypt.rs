use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    process, time,
};

use clap::Args;

use fencryption::{crypto::Crypto, walk_dir::WalkDir};

#[derive(Args)]
/// Decrypt specified encrypted file/directory using the passed
/// key
pub struct Command {
    /// Key used to decrypt
    #[clap(value_parser)]
    key: String,

    /// Paths of the encrypted file(s)/directory(ies) to
    /// decrypt
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

    if args.output_path.is_some() && args.paths.len() != 1 {
        println!("Error: Only one input path can be provided when setting an output path");
        process::exit(1);
    }

    // Runs for every provided input path
    for input_path in &args.paths {
        let input_path = PathBuf::from(input_path);
        let output_path = match &args.output_path {
            Some(v) => PathBuf::from(v),
            None => {
                let mut path = PathBuf::from(&input_path);
                path.set_extension("dec");
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
                    _ => {
                        println!("Error: Failed to create base directory");
                        if args.debug == true {
                            println!("  {}", e)
                        }
                        process::exit(1);
                    }
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
                            e => {
                                println!("Error: Failed to create sub-directory");
                                if args.debug == true {
                                    println!("  {}", e)
                                }
                                process::exit(1);
                            }
                        };
                    };
                } else if entry_type.is_file() {
                    print!("{} ... ", entry_path.display());

                    let mut source = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&entry_path)
                        .unwrap_or_else(|e| {
                            println!("ERROR");
                            println!("Error: Failed to read source file");
                            if args.debug == true {
                                println!("  {}", e)
                            }
                            process::exit(1);
                        });
                    let mut dest = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(&new_entry_path)
                        .unwrap_or_else(|e| {
                            println!("ERROR");
                            println!("Error: Failed to read/create destination file");
                            if args.debug == true {
                                println!("  {}", e)
                            }
                            process::exit(1);
                        });

                    match crypto.decrypt_stream(&mut source, &mut dest) {
                        Ok(_) => println!("Ok"),
                        Err(e) => {
                            println!("ERROR");
                            println!("Error: Failed to decrypt");
                            if args.debug == true {
                                println!("  {}", e)
                            }
                            process::exit(1);
                        }
                    };
                } else {
                    println!("{} ... SKIPPED (unknown type)", entry_path.display());
                };
            }
        } else if entry_metadata.file_type().is_file() {
            // The case where the entry is a file

            print!("{} ... ", input_path.display());

            let mut source = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&input_path)
                .unwrap_or_else(|e| {
                    println!("ERROR");
                    println!("Error: Failed to read source file");
                    if args.debug == true {
                        println!("  {}", e)
                    }
                    process::exit(1);
                });
            let mut dest = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&output_path)
                .unwrap_or_else(|e| {
                    println!("ERROR");
                    println!("Error: Failed to read/create destination file");
                    if args.debug == true {
                        println!("  {}", e)
                    }
                    process::exit(1);
                });

            match crypto.decrypt_stream(&mut source, &mut dest) {
                Ok(_) => println!("Ok"),
                Err(e) => {
                    println!("ERROR");
                    println!("Error: Failed to decrypt");
                    if args.debug == true {
                        println!("  {}", e)
                    }
                    process::exit(1);
                }
            };
        } else {
            // The case where the entry is something else
            println!("{} ... SKIPPED (unknown type)", input_path.display());
        }
    }

    println!(
        "\nDone: All Ok ({}ms elapsed)",
        timer.elapsed().unwrap_or_default().as_millis()
    )
}
