use clap::Parser;

use crate::commands::Commands;

mod commands;

#[derive(Parser)]
#[clap(name = "fencryption", author, version)]
#[clap(about = "A simple program to encrypt/decrypt files and directories.", long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ping(_) => commands::ping::action(),
        Commands::Encrypt(args) => commands::encrypt::action(args),
        Commands::Decrypt(args) => commands::decrypt::action(args),
    }
}
