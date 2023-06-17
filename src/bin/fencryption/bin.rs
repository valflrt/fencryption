use clap::Parser;
use fencryption_lib::log;

use crate::{
    cli::Cli,
    commands::{decrypt_file, decrypt_text, encrypt_file, encrypt_text},
    error::Result,
    logic::prompt_key,
};

mod cli;
mod commands;
mod error;
mod logic;
mod text;
mod warn;

#[cfg(test)]
mod tests;

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
                let key = prompt_key(true)?;

                let output = encrypt_file(key, paths, output_path, *overwrite, *delete_original)?;

                logic::log_stats(output, *debug);
            }
            cli::EncryptCommands::Text { text } => {
                let key = prompt_key(true)?;

                let output = encrypt_text(key, text)?;

                logic::log_stats(output, false);
            }
        },
        cli::Commands::Decrypt { command } => match command {
            cli::DecryptCommands::File {
                paths,
                output_path,
                overwrite,
                delete_original,
                debug,
            } => {
                let key = prompt_key(false)?;

                let output = decrypt_file(key, paths, output_path, *overwrite, *delete_original)?;

                logic::log_stats(output, *debug);
            }
            cli::DecryptCommands::Text { encrypted } => {
                let key = prompt_key(false)?;

                let output = decrypt_text(key, encrypted)?;

                logic::log_stats(output, false);
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
