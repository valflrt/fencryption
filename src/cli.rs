pub mod log;
pub mod util;

use clap::{command, Parser};

use crate::commands;

use self::util::{handle_error, handle_success};

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
        commands::CommandEnum::Encrypt(args) => match commands::encrypt::action(args) {
            Ok(m) => handle_success(m),
            Err(e) => handle_error(e),
        },
        commands::CommandEnum::Decrypt(args) => match commands::decrypt::action(args) {
            Ok(m) => handle_success(m),
            Err(e) => handle_error(e),
        },
        commands::CommandEnum::Pack(args) => match commands::pack::action(args) {
            Ok(m) => handle_success(m),
            Err(e) => handle_error(e),
        },
        commands::CommandEnum::Unpack(args) => match commands::unpack::action(args) {
            Ok(m) => handle_success(m),
            Err(e) => handle_error(e),
        },
    }
}
