use clap::Parser;
use cli::Cli;
use fencryption_lib::{
    commands::{encrypt_file, ErrorBuilder, Result},
    log,
};

use crate::logic::prompt_key;

mod cli;
mod error;
mod logic;
mod result;

// #[cfg(test)]
// mod tests;

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
                    ErrorBuilder::new()
                        .message("Failed to read key")
                        .error(e)
                        .build()
                })?;

                log::println_info("Starting encryption...");

                let (success, failures, skips, elapsed) =
                    encrypt_file::execute(key, paths, output_path, overwrite, delete_original)?;

                logic::log_stats(
                    success,
                    failures,
                    skips,
                    elapsed,
                    *debug,
                    logic::Command::Encrypt,
                );
            }
            cli::EncryptCommands::Text { text } => todo!(),
        },
        cli::Commands::Decrypt { command } => match command {
            cli::DecryptCommands::File {
                paths,
                output_path,
                overwrite,
                delete_original,
                debug,
            } => todo!(),
            cli::DecryptCommands::Text { encrypted } => todo!(),
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
