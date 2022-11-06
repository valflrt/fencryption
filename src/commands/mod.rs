use clap::Subcommand;

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
