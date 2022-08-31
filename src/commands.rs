use clap::Subcommand;

pub mod decrypt;
pub mod encrypt;
pub mod ping;

#[derive(Subcommand)]
pub enum Commands {
    Ping(ping::Args),
    Encrypt(encrypt::Args),
    Decrypt(decrypt::Args),
}
