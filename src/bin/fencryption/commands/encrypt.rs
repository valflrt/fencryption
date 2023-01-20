use std::{path::PathBuf, time};

use clap::{arg, Args};
use fencryption_lib::{crypto::Crypto, walk_dir::WalkDir};
use human_duration::human_duration;
use rpassword::{self, prompt_password};

use crate::{error::ErrorBuilder, log, result::Result};

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
    if args.paths.len() == 0 {
        return Err(ErrorBuilder::new()
            .error_message("Please provide at least one path")
            .debug_mode(args.debug)
            .build());
    }

    if args.paths.iter().any(|p| !p.exists()) {
        return Err(ErrorBuilder::new()
            .error_message("I can't work with files that don't exist")
            .debug_mode(args.debug)
            .build());
    }

    if args.output_path.is_some() {
        if args.paths.len() != 1 {
            return Err(ErrorBuilder::new()
                .debug_message("Only one input path can be provided when setting an output path")
                .debug_mode(args.debug)
                .build());
        }
        if args.output_path.as_ref().unwrap().exists() {
            return Err(ErrorBuilder::new()
                .error_message(
                    "The specified output path leads to a file or a directory, please remove it",
                )
                .debug_mode(args.debug)
                .build());
        }
    }

    let key = prompt_password(log::format_info("Enter key: ")).map_err(|e| {
        ErrorBuilder::new()
            .error_message("Failed to read key")
            .debug_error(e)
            .debug_mode(args.debug)
            .build()
    })?;
    let confirm_key = prompt_password(log::format_info("Confirm key: ")).map_err(|e| {
        ErrorBuilder::new()
            .error_message("Failed to read confirm key")
            .debug_error(e)
            .debug_mode(args.debug)
            .build()
    })?;

    if key.ne(&confirm_key) {
        return Err(ErrorBuilder::new()
            .error_message("The two keys don't match")
            .debug_mode(args.debug)
            .build());
    }

    if key.len() < 1 {
        return Err(ErrorBuilder::new()
            .error_message("You must set a key")
            .debug_mode(args.debug)
            .build());
    }

    log::println_info("Encrypting...");

    let crypto = Crypto::new(key);

    let timer = time::SystemTime::now();

    let success: Vec<PathBuf> = Vec::new();
    let failures: Vec<PathBuf> = Vec::new();
    let skips: Vec<PathBuf> = Vec::new();

    for input_path in args.paths {
        if input_path.is_dir() {
            let walk_dir = WalkDir::new(&input_path);
        } else if input_path.is_file() {
        } else {
        }
    }

    let elapsed = timer.elapsed().map_err(|e| {
        ErrorBuilder::new()
            .error_message("Failed to get elapsed time")
            .debug_mode(args.debug)
            .build()
    })?;

    if !success.is_empty() {
        log::println_success(format!(
            "Encrypted {} files in {}",
            success.len(),
            human_duration(&elapsed)
        ));
        if args.debug {
            success.iter().for_each(|msg| {
                log::println_success(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }
    if !failures.is_empty() {
        log::println_error(format!("Failed to encrypt {} files", failures.len()));
        if args.debug {
            failures.iter().for_each(|msg| {
                log::println_error(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }
    if !skips.is_empty() {
        log::println_info(format!(
            "{} entr{} were skipped (unknown type)",
            skips.len(),
            if skips.len() == 1 { "y" } else { "ies" }
        ));
        if args.debug {
            skips.iter().for_each(|msg| {
                log::println_info(log::with_start_line(msg.to_str().unwrap(), "    "))
            });
        }
    }

    Ok(())
}
