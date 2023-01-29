use clap::{command, Parser};

use crate::{commands, log};

#[derive(Parser)]
#[command(name = "fencryption", version)]
/// A program to encrypt/decrypt files and directories
struct Cli {
    #[clap(subcommand)]
    command: commands::CommandEnum,

    /// Enable debug log
    #[clap(short = 'D', long, required = false, global = true)]
    debug: bool,
}

pub fn parse() {
    let parser = Cli::parse();

    if let Err(e) = commands::execute(parser.command) {
        log::println_error(e.to_string(parser.debug));
    };
}
