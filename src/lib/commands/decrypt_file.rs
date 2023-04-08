//! Decrypt file.

use std::{fs, iter::zip, path::PathBuf, sync::mpsc::channel, time};

use threadpool::ThreadPool;

use crate::{
    commands::{logic, Command, Error, ErrorBuilder, Result},
    crypto::Crypto,
    walk_dir::walk_dir,
};

/// Decrypts the given paths.
pub fn execute(
    key: &String,
    paths: &Vec<PathBuf>,
    output_path: &Option<PathBuf>,
    overwrite: &bool,
    delete_original: &bool,
) -> Result<(
    u32,
    Vec<(PathBuf, Error)>,
    Vec<(PathBuf, Error)>,
    time::Duration,
)> {
    let timer = time::SystemTime::now();

    let output_paths = logic::get_output_paths(&paths, &output_path, Command::Decrypt);

    logic::checks(&paths, &output_path)?;
    logic::overwrite(&output_paths, *overwrite)?;

    let mut success: u32 = 0;
    let mut failures: Vec<(PathBuf, Error)> = Vec::new();
    let mut skips: Vec<(PathBuf, Error)> = Vec::new();

    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    for (main_input_path, main_output_path) in zip(paths.to_owned(), output_paths) {
        match main_input_path.to_owned() {
            dir_path if dir_path.is_dir() => {
                fs::create_dir(&main_output_path).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to create output directory")
                        .error(e)
                        .build()
                })?;

                let walk_dir = walk_dir(&dir_path).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to read directory")
                        .error(e)
                        .build()
                })?;

                let threadpool = ThreadPool::new(8);
                let (tx, rx) = channel();
                let mut tries_n = 0;

                for dir_entry in walk_dir {
                    let entry = dir_entry.map_err(|e| {
                        ErrorBuilder::new()
                            .message("Failed to read directory entry")
                            .error(e)
                            .build()
                    })?;
                    let input_path = entry.path();

                    match input_path {
                        input_path if input_path.is_file() => {
                            tries_n += 1;

                            let crypto = crypto.clone();
                            let tx = tx.clone();
                            let output_dir_path = main_output_path.clone();

                            threadpool.execute(move || {
                                let result = logic::decrypt_file(
                                    crypto,
                                    &input_path,
                                    logic::OutputDecPath::Parent(output_dir_path),
                                );
                                tx.send((input_path.to_owned(), result)).unwrap();
                            });
                        }
                        input_path => {
                            if !input_path.is_dir() {
                                skips.push((
                                    input_path,
                                    ErrorBuilder::new().message("Unknown entry type").build(),
                                ));
                            }
                        }
                    }
                }

                threadpool.join();
                rx.iter()
                    .take(tries_n)
                    .for_each(|(path, result)| match result {
                        Ok(_) => success += 1,
                        Err(e) => failures.push((path, e)),
                    })
            }
            path if path.is_file() => {
                match logic::decrypt_file(
                    crypto.to_owned(),
                    &path,
                    logic::OutputDecPath::Direct(main_output_path.to_owned()),
                ) {
                    Ok(_) => success += 1,
                    Err(e) => failures.push((path, e)),
                };
            }
            path => skips.push((
                path,
                ErrorBuilder::new().message("Unknown entry type").build(),
            )),
        }

        logic::delete_original(&main_input_path, *delete_original)?;
    }

    Ok((
        success,
        failures,
        skips,
        timer.elapsed().map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to get elapsed time")
                .error(e)
                .build()
        })?,
    ))
}
