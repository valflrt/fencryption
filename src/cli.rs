use clap::{command, Parser};

use crate::{actions::ActionError, commands, log};

/// Handles command action error
pub fn handle_error(error: ActionError) {
    log::println_error(error.message());
    if let Some(d) = error.debug_message() {
        log::println_error(log::with_start_line(d, "    "));
    };
    quit::with_code(1);
}

#[derive(Parser)]
#[command(name = "fencryption", version)]
/// A program to encrypt/decrypt files and full directories
struct Cli {
    #[clap(subcommand)]
    command: commands::CommandEnum,

    /// Enable debug log
    #[clap(short = 'D', long, required = false, global = true)]
    debug: bool,
}

pub fn parse() {
    match &Cli::parse().command {
        commands::CommandEnum::Encrypt(args) => {
            if let Err(e) = commands::encrypt::execute(args) {
                handle_error(e);
            }
        }
        commands::CommandEnum::Decrypt(args) => {
            if let Err(e) = commands::decrypt::execute(args) {
                handle_error(e);
            }
        }
        commands::CommandEnum::Pack(args) => {
            if let Err(e) = commands::pack::execute(args) {
                handle_error(e);
            }
        }
        commands::CommandEnum::Unpack(args) => {
            if let Err(e) = commands::unpack::execute(args) {
                handle_error(e);
            }
        }
    }
}
