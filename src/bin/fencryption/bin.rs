use clap::Parser;
use cli::Cli;
use fencryption_lib::{
    commands::{
        decrypt_file, decrypt_text, encrypt_file, encrypt_text, Command, ErrorBuilder, Result,
    },
    log,
};

use crate::logic::prompt_key;

mod cli;
mod logic;

fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        cli::Commands::Encrypt { command } => match command {
            cli::EncryptCommands::File {
                paths,
                output_path,
                overwrite,
                delete_original,
                debug,
            } => {
                let key = prompt_key(true).map_err(|e| {
                    ErrorBuilder::default()
                        .message("Failed to read key")
                        .error(e)
                        .build()
                })?;

                log::println_info("Starting encryption...");

                let (success, failures, skips, elapsed) =
                    encrypt_file::execute(&key, paths, output_path, overwrite, delete_original)?;

                util::log_stats(output, *debug);
                logic::log_stats(
                    success,
                    failures,
                    skips,
                    elapsed,
                    *debug,
                    Command::EncryptFile,
                );
            }
            cli::EncryptCommands::Text { text } => {
                let key = prompt_key(true).map_err(|e| {
                    ErrorBuilder::default()
                        .message("Failed to read key")
                        .error(e)
                        .build()
                })?;
                let enc = encrypt_text::execute(&key, text)?;
                log::println_success(format!("Successfully encrypted text: base64 {}", enc));
            }
        },
        cli::Commands::Decrypt { command } => match command {
            cli::DecryptCommands::File {
                paths,
                output_path,
                overwrite,
                delete_encrypted,
                debug,
            } => {
                let key = prompt_key(false).map_err(|e| {
                    ErrorBuilder::default()
                        .message("Failed to read key")
                        .error(e)
                        .build()
                })?;

                log::println_info("Starting decryption...");

                let (success, failures, skips, elapsed) =
                    decrypt_file::execute(&key, paths, output_path, overwrite, delete_encrypted)?;

                logic::log_stats(
                    success,
                    failures,
                    skips,
                    elapsed,
                    *debug,
                    Command::EncryptFile,
                );
            }
            cli::DecryptCommands::Text { encrypted } => {
                let key = prompt_key(false).map_err(|e| {
                    ErrorBuilder::default()
                        .message("Failed to read key")
                        .error(e)
                        .build()
                })?;
                let dec = decrypt_text::execute(&key, encrypted)?;
                log::println_success(format!("Successfully decrypted text: \"{}\"", dec));
            }
        },
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        log::println_error(e.to_string(cli.debug));
    };
}
