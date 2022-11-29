use clap::{command, Parser};

use crate::{actions::ActionError, commands, log};

/// Handles command action error
pub fn handle_error(error: ActionError, debug: bool) {
    log::println_error(error.message());
    if debug {
        if let Some(d) = error.debug_message() {
            println!("{}", d);
            log::println_error(log::with_start_line(d, "    "));
        }
    }
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
                handle_error(e, args.debug);
            }
        }
        commands::CommandEnum::Decrypt(args) => {
            if let Err(e) = commands::decrypt::execute(args) {
                handle_error(e, args.debug);
            }
        }
        commands::CommandEnum::Pack(args) => {
            if let Err(e) = commands::pack::execute(args) {
                handle_error(e, args.debug);
            }
        }
        commands::CommandEnum::Unpack(args) => {
            if let Err(e) = commands::unpack::execute(args) {
                handle_error(e, args.debug);
            }
        }
    }
}
