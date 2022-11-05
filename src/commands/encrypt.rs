use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    process, time,
};

use clap::{arg, Args};
use human_duration::human_duration;
use threadpool::ThreadPool;

use fencryption::{crypto::Crypto, walk_dir::WalkDir};

#[derive(Args, Clone)]
/// Encrypt specified file/directory using the passed key
pub struct Command {
    /// Key used to encrypt
    key: String,

    /// Paths of the file(s)/directory(ies) to encrypt
    paths: Vec<PathBuf>,

    /// Set output path (only supported when one input path
    /// provided)
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// Whether to overwrite the output file/directory
    #[clap(short = 'O', long)]
    overwrite: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) {
    let timer = time::SystemTime::now();
    let mut counter: u128 = 0;

    if args.output_path.is_some() && args.paths.len() != 1 {
        println!("Error: Only one input path can be provided when setting an output path");
        process::exit(1);
    }

    let crypto = Crypto::new(args.key.as_bytes()).unwrap_or_else(|e| {
        println!("Error: Failed to create cipher");
        if args.debug == true {
            println!("  - {:?}", e)
        }
        process::exit(1);
    });

    // Runs for every provided input path.
    for input_path in &args.paths {
        let output_path = match &args.output_path {
            Some(v) => v.to_owned(),
            None => {
                let mut path = input_path.clone();
                path.set_file_name(
                    [
                        path.file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                        ".enc",
                    ]
                    .concat(),
                );
                path
            }
        };

        if input_path.exists() == false {
            println!("Error: The item pointed by the given path doesn't exist");
            process::exit(1);
        }

        if output_path.exists() == true && args.overwrite == false {
            println!(
                "Error: The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)"
            );
            process::exit(1);
        }

        // Reads entry metadata to act in consequence.
        let entry_metadata = fs::metadata(input_path).unwrap_or_else(|e| {
            println!("Error: Failed to read entry metadata");
            if args.debug == true {
                println!("  - {:?}", e)
            }
            process::exit(1);
        });
        if entry_metadata.file_type().is_dir() {
            // The case where the entry is a directory.

            // Creates base directory to put encrypted files
            // in.
            if let Err(e) = fs::create_dir(&output_path) {
                match e.kind() {
                    io::ErrorKind::AlreadyExists => (),
                    _ => {
                        println!("Error: Failed to create base directory");
                        if args.debug == true {
                            println!("  - {:?}", e)
                        }
                        process::exit(1);
                    }
                };
            };

            let walk_dir = WalkDir::new(input_path).unwrap_or_else(|e| {
                println!("Error: Failed to read directory");
                if args.debug == true {
                    println!("  - {:?}", e)
                }
                process::exit(1);
            });

            let threadpool = ThreadPool::new(8);

            // Runs for every entry in the specified directory.
            for entry in walk_dir {
                let crypto = crypto.clone();

                let args = args.clone();

                let entry = entry.unwrap_or_else(|e| {
                    println!("Error: Failed to read entry");
                    if args.debug == true {
                        println!("  - {:?}", e)
                    }
                    process::exit(1);
                });
                let entry_path = entry.path();
                let new_entry_path =
                    output_path.join(entry_path.strip_prefix(input_path).unwrap_or_else(|e| {
                        println!("\nError: Failed to establish relative file path");
                        if args.debug == true {
                            println!("  - {:?}", e)
                        }
                        process::exit(1);
                    }));

                // Reads entry type to act depending on it.
                let entry_type = entry.file_type().unwrap_or_else(|e| {
                    println!("Error: Failed to read file type");
                    if args.debug == true {
                        println!("  - {:?}", e)
                    }
                    process::exit(1);
                });
                if entry_type.is_dir() {
                    if let Err(e) = fs::create_dir(&new_entry_path) {
                        match e.kind() {
                            io::ErrorKind::AlreadyExists => (),
                            e => {
                                println!("Error: Failed to create sub-directory");
                                if args.debug == true {
                                    println!("  - {:?}", e)
                                }
                                process::exit(1);
                            }
                        };
                    };
                } else if entry_type.is_file() {
                    counter += 1;
                    threadpool.execute(move || {
                        let mut source = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(&entry_path)
                            .unwrap_or_else(|e| {
                                println!("[ERROR] {}", entry.path().display());
                                println!("\nError: Failed to read source file");
                                if args.debug == true {
                                    println!("  - {:?}", e)
                                }
                                process::exit(1);
                            });
                        let mut dest = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(&new_entry_path)
                            .unwrap_or_else(|e| {
                                println!("[ERROR] {}", entry.path().display());
                                println!("\nError: Failed to read/create destination file");
                                if args.debug == true {
                                    println!("  - {:?}", e)
                                }
                                process::exit(1);
                            });

                        match crypto.encrypt_stream(&mut source, &mut dest) {
                            Ok(_) => println!("[OK] {}", entry.path().display()),
                            Err(e) => {
                                println!("[ERROR] {}", entry.path().display());
                                println!("\nError: Failed to encrypt");
                                if args.debug == true {
                                    println!("  - {:?}", e)
                                }
                                process::exit(1);
                            }
                        };
                    });
                } else {
                    println!("[SKIPPED] {} (unknown type)", entry_path.display());
                };
            }

            threadpool.join();
        } else if entry_metadata.file_type().is_file() {
            // The case where the entry is a file.
            let mut source = OpenOptions::new()
                .read(true)
                .write(true)
                .open(input_path)
                .unwrap_or_else(|e| {
                    println!("[ERROR] {}", input_path.display());
                    println!("\nError: Failed to read source file");
                    if args.debug == true {
                        println!("  - {:?}", e)
                    }
                    process::exit(1);
                });
            let mut dest = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&output_path)
                .unwrap_or_else(|e| {
                    println!("[ERROR] {}", input_path.display());
                    println!("\nError: Failed to read/create destination file");
                    if args.debug == true {
                        println!("  - {:?}", e)
                    }
                    process::exit(1);
                });

            match crypto.encrypt_stream(&mut source, &mut dest) {
                Ok(_) => {
                    println!("[OK] {}", input_path.display());
                    counter += 1;
                }
                Err(e) => {
                    println!("[ERROR] {}", input_path.display());
                    println!("\nError: Failed to encrypt");
                    if args.debug == true {
                        println!("  - {:?}", e)
                    }
                    process::exit(1);
                }
            };
        } else {
            // The case where the entry is something else.
            println!("{} ... SKIPPED (unknown type)", input_path.display());
        }
    }

    println!(
        "\nDone: Encrypted {} files in {}",
        counter,
        human_duration(&timer.elapsed().unwrap_or_default())
    )
}
