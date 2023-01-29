use clap::Subcommand;

use crate::error::Error;

pub mod decrypt;
pub mod encrypt;

#[derive(Subcommand)]
pub enum CommandEnum {
    Encrypt(encrypt::Command),
    Decrypt(decrypt::Command),
}

pub fn execute(command: CommandEnum) -> Result<(), Error> {
    match command {
        CommandEnum::Encrypt(c) => encrypt::execute(&c),
        CommandEnum::Decrypt(c) => decrypt::execute(&c),
    }
}
