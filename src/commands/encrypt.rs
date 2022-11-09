use std::{
    fs::{self, OpenOptions},
    io,
    path::PathBuf,
    time,
};

use clap::{arg, Args};
use human_duration::human_duration;
use rpassword::{self, prompt_password};
use threadpool::ThreadPool;

use crate::cli::{
    log,
    util::{ActionError, ActionResult},
};
use fencryption::{crypto::Crypto, walk_dir::WalkDir};

#[derive(Args, Clone)]
/// Encrypt specified file/directory using the passed key
pub struct Command {
    /// Paths of the file(s)/directory(ies) to encrypt
    #[arg(required = true)]
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

pub fn action(args: &Command) -> ActionResult {
    let mut counter: u128 = 0;

    if args.paths.len() == 0 {
        return Err(ActionError::new("You must provide at least one path", None));
    }

    if args.output_path.is_some() && args.paths.len() != 1 {
        return Err(ActionError::new(
            "Only one input path can be provided when setting an output path",
            None,
        ));
    };

    let key = prompt_password(log::format_info("Enter key: "))
        .map_err(|e| ActionError::new("Failed to read key", Some(format!("  - {:?}", e))))?;
    let confirm_key = prompt_password(log::format_info("Confirm key: ")).map_err(|e| {
        ActionError::new("Failed to read confirm key", Some(format!("  - {:?}", e)))
    })?;

    if key.ne(&confirm_key) {
        return Err(ActionError::new("The two keys don't match", None));
    };

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
            None,
        ));
    };

    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| ActionError::new("Failed to create cipher", Some(format!("  - {:?}", e))))?;

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
            return Err(ActionError::new(
                "The item pointed by the given path doesn't exist",
                None,
            ));
        };

        if output_path.exists() == true && args.overwrite == false {
            return Err(ActionError::new(
                "The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)",
                None,
            ));
        };

        if input_path.is_dir() {
            // The case where the entry is a directory.

            // Creates base directory to put encrypted files
            // in.
            if let Err(e) = fs::create_dir(&output_path) {
                match e.kind() {
                    io::ErrorKind::AlreadyExists => (),
                    _ => {
                        return Err(ActionError::new(
                            "Failed to create base directory",
                            Some(format!("  - {:?}", e)),
                        ));
                    }
                };
            };

            let walk_dir = WalkDir::new(input_path).map_err(|e| {
                ActionError::new("Failed to read directory", Some(format!("  - {:?}", e)))
            })?;

            let threadpool = ThreadPool::new(8);

            // Runs for every entry in the specified directory.
            for entry in walk_dir {
                let crypto = crypto.clone();

                let entry = entry.map_err(|e| {
                    ActionError::new("Failed to read entry", Some(format!("  - {:?}", e)))
                })?;
                let entry_path = entry.path();
                let new_entry_path =
                    output_path.join(entry_path.strip_prefix(input_path).map_err(|e| {
                        ActionError::new("Couldn't find output path", Some(format!("  - {:?}", e)))
                    })?);

                if entry_path.is_dir() {
                    if let Err(e) = fs::create_dir(&new_entry_path) {
                        match e.kind() {
                            io::ErrorKind::AlreadyExists => (),
                            e => {
                                return Err(ActionError::new(
                                    "Failed to create sub-directory",
                                    Some(format!("  - {:?}", e)),
                                ));
                            }
                        };
                    };
                } else if entry_path.is_file() {
                    counter += 1;
                    threadpool.execute(move || {
                        let mut source = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(&entry_path)
                            .unwrap();
                        let mut dest = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(&new_entry_path)
                            .unwrap();

                        match crypto.encrypt_stream(&mut source, &mut dest) {
                            Ok(_) => log::println_success(format!("{} Ok", entry.path().display())),
                            Err(_) => panic!(),
                        };
                    });
                } else {
                    log::println_info(format!(
                        "Skipped {} (unknown entry type)",
                        entry_path.display()
                    ));
                };
            }

            threadpool.join();
            if threadpool.panic_count() != 0 {
                log::println_error(format!(
                    "Failed to encrypt {} entries",
                    threadpool.panic_count()
                ));
            };
        } else if input_path.is_file() {
            // The case where the entry is a file.
            let mut source = OpenOptions::new()
                .read(true)
                .write(true)
                .open(input_path)
                .map_err(|e| {
                    ActionError::new("Failed to read source file", Some(format!("  - {:?}", e)))
                })?;
            let mut dest = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&output_path)
                .map_err(|e| {
                    ActionError::new(
                        "Failed to read/create destination file",
                        Some(format!("  - {:?}", e)),
                    )
                })?;

            match crypto.encrypt_stream(&mut source, &mut dest) {
                Ok(_) => {
                    log::println_success(format!("{} Ok", input_path.display()));
                    counter += 1;
                }
                Err(e) => {
                    return Err(ActionError::new(
                        "Failed to encrypt",
                        Some(format!("  - {:?}", e)),
                    ));
                }
            };
        } else {
            // The case where the entry is something else.
            log::println_info(format!(
                "Skipped {} (unknown entry type)",
                input_path.display()
            ));
        };
    }

    Ok(Some(format!(
        "Encrypted {} files in {}",
        counter,
        human_duration(&timer.elapsed().unwrap_or_default())
    )))
}
