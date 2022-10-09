use clap::Subcommand;

pub mod decrypt;
pub mod encrypt;

#[derive(Subcommand)]
pub enum CommandEnum {
    Encrypt(encrypt::Command),
    Decrypt(decrypt::Command),
}
