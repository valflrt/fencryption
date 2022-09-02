use clap::Parser;

mod commands;

#[derive(Parser)]
#[clap(name = "fencryption", version)]
/// A program to encrypt/decrypt files and full directories
struct Cli {
    #[clap(subcommand)]
    command: commands::Commands,

    /// Enables debug log
    #[clap(
        short = 'D',
        long,
        value_parser,
        default_value_t = false,
        global = true
    )]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        commands::Commands::Encrypt(args) => commands::encrypt::action(args),
        commands::Commands::Decrypt(args) => commands::decrypt::action(args),
    }
}
