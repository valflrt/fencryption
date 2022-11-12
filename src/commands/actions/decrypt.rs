use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    sync::mpsc::channel,
    time::{self, Duration},
};

use threadpool::ThreadPool;

use fencryption_lib::{crypto::Crypto, walk_dir::WalkDir};

use crate::cli::{CommandError, CommandResult};

pub fn decrypt(
    input_paths: Vec<PathBuf>,
    output_path: Option<PathBuf>,
    key: String,
    overwrite: bool,
) -> CommandResult<(Duration, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let mut success_paths: Vec<PathBuf> = Vec::new();
    let mut skipped_paths: Vec<PathBuf> = Vec::new();
    let mut failed_paths: Vec<PathBuf> = Vec::new();
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| CommandError::new("Failed to create cipher", Some(format!("{:#?}", e))))?;

    // Runs for every provided input path.
    for input_path in input_paths {
        let output_path = match &output_path {
            Some(v) => v.to_owned(),
            None => {
                let mut path = input_path.to_owned();
                path.set_file_name(
                    [
                        path.file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                        ".dec",
                    ]
                    .concat(),
                );
                path
            }
        };

        if input_path.exists() == false {
            return Err(CommandError::new(
                "The item pointed by the given path doesn't exist",
                None,
            ));
        };

        if output_path.exists() == true && overwrite == false {
            return Err(CommandError::new(
                "The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)",
                None,
            ));
        };

        if input_path.is_dir() {
            // The case where the entry is a directory.

            // Creates base directory to put encrypted files
            // in.
            fs::create_dir(&output_path).ok();

            let walk_dir = WalkDir::new(&input_path).iter().map_err(|e| {
                CommandError::new("Failed to read directory", Some(format!("{:#?}", e)))
            })?;

            let threadpool = ThreadPool::new(8);
            let (tx, rx) = channel();
            let mut tries_nb = 0;

            // Runs for every entry in the specified directory.
            for entry in walk_dir {
                let crypto = crypto.clone();

                let entry = entry.map_err(|e| {
                    CommandError::new("Failed to read entry", Some(format!("{:#?}", e)))
                })?;
                let entry_path = entry.path();
                let new_entry_path =
                    output_path.join(entry_path.strip_prefix(&input_path).map_err(|e| {
                        CommandError::new("Couldn't find output path", Some(format!("{:#?}", e)))
                    })?);

                if entry_path.is_dir() {
                    fs::create_dir(&new_entry_path).ok();
                } else if entry_path.is_file() {
                    tries_nb += 1;
                    let tx = tx.clone();
                    let entry_path = entry_path.clone();
                    threadpool.execute(move || {
                        let mut source =
                            match OpenOptions::new().read(true).write(true).open(&entry_path) {
                                Ok(v) => v,
                                Err(_) => {
                                    tx.send((entry_path, false)).unwrap();
                                    return;
                                }
                            };
                        let mut dest = match OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(&new_entry_path)
                        {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };

                        if let Err(_) = crypto.decrypt_stream(&mut source, &mut dest) {
                            tx.send((entry_path, false)).unwrap();
                            return;
                        }

                        tx.send((entry_path, true)).unwrap();
                    });
                } else {
                    skipped_paths.push(entry_path.to_owned());
                }
            }

            threadpool.join();
            rx.iter().take(tries_nb).for_each(|(path, success)| {
                if success {
                    success_paths.push(path);
                } else {
                    failed_paths.push(path);
                }
            })
        } else if input_path.is_file() {
            // The case where the entry is a file.
            let mut source = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&input_path)
                .map_err(|e| {
                    CommandError::new("Failed to read source file", Some(format!("{:#?}", e)))
                })?;
            let mut dest = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&output_path)
                .map_err(|e| {
                    CommandError::new(
                        "Failed to read/create destination file",
                        Some(format!("{:#?}", e)),
                    )
                })?;

            match crypto.decrypt_stream(&mut source, &mut dest) {
                Ok(_) => success_paths.push(input_path.to_owned()),
                Err(_) => failed_paths.push(input_path.to_owned()),
            };
        } else {
            skipped_paths.push(input_path);
        };
    }

    Ok((
        timer.elapsed().unwrap_or_default(),
        success_paths,
        skipped_paths,
        failed_paths,
    ))
}
