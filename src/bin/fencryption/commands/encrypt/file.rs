use std::{fs, iter::zip, path::PathBuf, sync::mpsc::channel, time};

use clap::{arg, Args};
use fencryption_lib::{crypto::Crypto, walk_dir::walk_dir};
use threadpool::ThreadPool;
use uuid::Uuid;

use crate::{
    error::{Error, ErrorBuilder},
    log, logic,
    result::Result,
};

#[derive(Args, Clone)]
/// Encrypt files and directories
pub struct Command {
    /// Paths of files and directories to encrypt
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Set output path (only supported when one input path
    /// is provided)
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// Overwrite output files and directories
    #[clap(short = 'O', long)]
    overwrite: bool,

    /// Delete original files and directories after encrypting
    #[clap(short = 'd', long)]
    delete_original: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn execute(args: &Command) -> Result<()> {
    let output_paths =
        logic::get_output_paths(&args.paths, &args.output_path, logic::Command::Encrypt);

    logic::checks(&args.paths, &args.output_path)?;
    logic::overwrite(&output_paths, args.overwrite)?;

    let mut success: Vec<PathBuf> = Vec::new();
    let mut failures: Vec<(PathBuf, Error)> = Vec::new();
    let mut skips: Vec<(PathBuf, Error)> = Vec::new();

    let key = logic::prompt_key(true)?;
    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    log::println_info("Encrypting...");

    let timer = time::SystemTime::now();
    for (main_input_path, main_output_path) in zip(args.paths.to_owned(), output_paths) {
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
                let mut tries_nb = 0;

                for dir_entry in walk_dir {
                    let entry = dir_entry.map_err(|e| {
                        ErrorBuilder::new()
                            .message("Failed to read directory entry")
                            .error(e)
                            .build()
                    })?;
                    let input_path = entry.path();
                    let relative_entry_path = input_path
                        .strip_prefix(&dir_path)
                        .map_err(|e| {
                            ErrorBuilder::new()
                                .message("Failed to get relative entry path")
                                .error(e)
                                .build()
                        })?
                        .to_owned();
                    let output_path = main_output_path.join(Uuid::new_v4().to_string());

                    match input_path {
                        input_path if input_path.is_file() => {
                            tries_nb += 1;

                            let crypto = crypto.clone();
                            let tx = tx.clone();
                            let output_path = output_path.clone();

                            threadpool.execute(move || {
                                let result = logic::encrypt_file(
                                    crypto,
                                    &input_path,
                                    &output_path,
                                    Some(relative_entry_path.to_owned()),
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
                    .take(tries_nb)
                    .for_each(|(path, result)| match result {
                        Ok(_) => success.push(path),
                        Err(e) => failures.push((path, e)),
                    })
            }
            path if path.is_file() => {
                match logic::encrypt_file(crypto.to_owned(), &path, &main_output_path, None) {
                    Ok(_) => success.push(path),
                    Err(e) => failures.push((path, e)),
                };
            }
            path => skips.push((
                path,
                ErrorBuilder::new().message("Unknown entry type").build(),
            )),
        }

        logic::delete_original(&main_input_path, args.delete_original)?;
    }

    logic::log_stats(
        success,
        failures,
        skips,
        timer.elapsed().map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to get elapsed time")
                .error(e)
                .build()
        })?,
        args.debug,
        logic::Command::Encrypt,
    );

    Ok(())
}
