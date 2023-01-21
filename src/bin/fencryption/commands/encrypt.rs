use std::{fs, path::PathBuf, sync::mpsc::channel, time};

use clap::{arg, Args};
use fencryption_lib::{crypto::Crypto, walk_dir::WalkDir};
use threadpool::ThreadPool;
use uuid::Uuid;

use crate::{
    error::{Error, ErrorBuilder},
    log,
    logic::{self},
    result::Result,
};

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

    /// Delete the original directory after encrypting
    #[clap(short = 'd', long)]
    delete_original: bool,

    #[clap(from_global)]
    pub debug: bool,
}

pub fn execute(args: &Command) -> Result<()> {
    logic::checks(&args.paths, &args.output_path)?;

    let key = logic::prompt_key(true)?;

    log::println_info("Encrypting...");

    let crypto = Crypto::new(key).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to initialize encryption utils")
            .error(e)
            .build()
    })?;

    let timer = time::SystemTime::now();

    let mut success: Vec<PathBuf> = Vec::new();
    let mut failures: Vec<(PathBuf, Error)> = Vec::new();
    let mut skips: Vec<(PathBuf, Error)> = Vec::new();

    for input_path in &args.paths {
        let output_path = match &args.output_path {
            Some(p) => p.to_owned(),
            None => logic::change_file_name(input_path, |n| [n, ".enc"].concat()),
        };

        logic::handle_already_existing_item(&output_path, args.overwrite)?;

        if input_path.is_dir() {
            let walk_dir = WalkDir::new(&input_path).iter().map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to read directory")
                    .error(e)
                    .build()
            })?;

            let threadpool = ThreadPool::new(8);
            let (tx, rx) = channel();
            let mut tries_nb = 0;

            fs::create_dir(&output_path).map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to create output directory")
                    .error(e)
                    .build()
            })?;

            for dir_entry in walk_dir {
                let entry = dir_entry.map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to read directory entry")
                        .error(e)
                        .build()
                })?;
                let entry_path = entry.path();
                let new_entry_path = output_path.join(Uuid::new_v4().to_string());

                if entry_path.is_file() {
                    tries_nb += 1;

                    let crypto = crypto.clone();
                    let tx = tx.clone();
                    let entry_path = entry_path.clone();

                    threadpool.execute(move || {
                        let result =
                            logic::encrypt_file(crypto, &entry_path, &new_entry_path, true);
                        tx.send((entry_path.to_owned(), result)).unwrap();
                    });
                } else if !entry_path.is_dir() {
                    skips.push((
                        entry_path,
                        ErrorBuilder::new().message("Unknown entry type").build(),
                    ));
                }
            }

            threadpool.join();
            rx.iter()
                .take(tries_nb)
                .for_each(|(path, result)| match result {
                    Ok(_) => success.push(path),
                    Err(e) => failures.push((path, e)),
                })
        } else if input_path.is_file() {
            match logic::encrypt_file(crypto.to_owned(), &input_path, &output_path, false) {
                Ok(_) => success.push(input_path.to_owned()),
                Err(e) => failures.push((input_path.to_owned(), e)),
            };
        } else {
            skips.push((
                input_path.to_owned(),
                ErrorBuilder::new().message("Unknown entry type").build(),
            ))
        }
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
