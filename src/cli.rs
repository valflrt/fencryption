use clap::Parser;

use crate::commands;

#[derive(Parser)]
#[clap(name = "fencryption", version)]
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
        commands::CommandEnum::Encrypt(args) => commands::encrypt::action(args),
        commands::CommandEnum::Decrypt(args) => commands::decrypt::action(args),
    }
}
