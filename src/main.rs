use clap::Parser;

use crate::commands::Commands;

mod commands;

#[derive(Parser)]
#[clap(name = "fencryption", version)]
/// A simple program to encrypt/decrypt files and directories
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Enables debug log
    #[clap(short = 'D', long, value_parser, default_value_t = false)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ping(_) => commands::ping::action(),
        Commands::Encrypt(args) => commands::encrypt::action(args),
        Commands::Decrypt(args) => commands::decrypt::action(args),
    }
}
