use clap::Subcommand;

use crate::error::Error;

pub mod decrypt;
pub mod encrypt;
pub mod pack;
pub mod unpack;

#[derive(Subcommand)]
pub enum CommandEnum {
    Encrypt(encrypt::Command),
    Decrypt(decrypt::Command),
    Pack(pack::Command),
    Unpack(unpack::Command),
}

pub fn execute(command: CommandEnum) -> Result<(), Error> {
    match command {
        CommandEnum::Encrypt(c) => encrypt::execute(&c),
        CommandEnum::Decrypt(c) => decrypt::execute(&c),
        CommandEnum::Pack(c) => pack::execute(&c),
        CommandEnum::Unpack(c) => unpack::execute(&c),
    }
}
