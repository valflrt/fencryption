use std::{path::PathBuf, time::Duration};

use fencryption_lib::{
    commands::{Command, Error, ErrorBuilder, Result},
    log,
};
use human_duration::human_duration;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PathMetadata(PathBuf);

pub fn prompt_key(double_check: bool) -> Result<String> {
    let key = prompt_password(log::format_info("Enter key: ")).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to read key")
            .error(e)
            .build()
    })?;

    if double_check {
        let confirm_key = prompt_password(log::format_info("Confirm key: ")).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read confirm key")
                .error(e)
                .build()
        })?;

        if key.ne(&confirm_key) {
            return Err(ErrorBuilder::new()
                .message("The two keys don't match")
                .build());
        };

        if key.len() < 1 {
            return Err(ErrorBuilder::new().message("You must set a key").build());
        };

        if key.len() < 6 {
            log::println_warn("Your key should have more than 6 characters to be more secure");
        };
    }

    Ok(key)
}

pub fn log_stats(
    success: u32,
    failures: Vec<(PathBuf, Error)>,
    skips: Vec<(PathBuf, Error)>,
    elapsed: Duration,
    debug: bool,
    command: Command,
) {
    log::println_success(format!(
        "{} {} file{} in {}",
        match command {
            Command::Encrypt => "Encrypted",
            Command::Decrypt => "Decrypted",
            // _ => panic!(),
        },
        success,
        if success == 1 { "" } else { "s" },
        human_duration(&elapsed)
    ));
    if !failures.is_empty() {
        log::println_error(format!(
            "Failed to {} {} file{}",
            match command {
                Command::Encrypt => "encrypt",
                Command::Decrypt => "decrypt",
                // _ => panic!(),
            },
            failures.len(),
            if failures.len() == 1 { "" } else { "s" }
        ));
        if debug {
            failures.iter().for_each(|v| {
                log::println_error(format!(
                    "{}\n{}",
                    log::with_start_line(v.0.to_str().unwrap_or_default(), "  "),
                    log::with_start_line(v.1.to_string(debug), "      "),
                ));
            });
        }
    }
    if !skips.is_empty() {
        log::println_info(format!(
            "{} entr{} were skipped (unknown type)",
            skips.len(),
            if skips.len() == 1 { "y" } else { "ies" }
        ));
        if debug {
            skips.iter().for_each(|v| {
                log::println_info(format!(
                    "{}\n{}",
                    log::with_start_line(v.0.to_str().unwrap_or_default(), "  "),
                    log::with_start_line(v.1.to_string(debug), "      "),
                ));
            });
        }
    }
}
