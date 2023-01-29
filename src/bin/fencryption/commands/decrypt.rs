use std::{fs, iter::zip, path::PathBuf, sync::mpsc::channel, time};

use clap::{arg, Args};
use fencryption_lib::{crypto::Crypto, walk_dir::WalkDir};
use threadpool::ThreadPool;

use crate::{
    error::{Error, ErrorBuilder},
    log, logic,
    result::Result,
};

#[derive(Args, Clone)]
/// Decrypt files and directories
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

    /// Delete original files and directories after decrypting
    #[clap(short = 'd', long)]
    delete_original: bool,

    #[clap(from_global)]
    pub debug: bool,
}

pub fn execute(args: &Command) -> Result<()> {
    let output_paths =
        logic::get_output_paths(&args.paths, &args.output_path, logic::Command::Decrypt);

    logic::checks(&args.paths, &args.output_path)?;
    logic::overwrite(&output_paths, args.overwrite)?;

    let mut success: Vec<PathBuf> = Vec::new();
    let mut failures: Vec<(PathBuf, Error)> = Vec::new();
    let mut skips: Vec<(PathBuf, Error)> = Vec::new();

    let key = logic::prompt_key(false)?;
    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    log::println_info("Decrypting...");

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

                let walk_dir = WalkDir::new(&dir_path).iter().map_err(|e| {
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

                    match input_path {
                        input_path if input_path.is_file() => {
                            tries_nb += 1;

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
                    .take(tries_nb)
                    .for_each(|(path, result)| match result {
                        Ok(_) => success.push(path),
                        Err(e) => failures.push((path, e)),
                    })
            }
            path if path.is_file() => {
                match logic::decrypt_file(
                    crypto.to_owned(),
                    &path,
                    logic::OutputDecPath::Direct(main_output_path.to_owned()),
                ) {
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
        logic::Command::Decrypt,
    );

    Ok(())
}
