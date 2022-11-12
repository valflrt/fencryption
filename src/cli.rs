pub mod log;
pub mod util;

use clap::{command, Parser};

use crate::{cli::util::handle_error, commands};

pub use crate::cli::util::*;

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
