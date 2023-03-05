use clap::{Args, Subcommand};

use crate::result::Result;

mod file;
mod text;

#[derive(Subcommand)]
pub enum CommandEnum {
    File(file::Command),
    Text(text::Command),
}

#[derive(Args)]
/// Encrypt text or files
pub struct Command {
    #[clap(subcommand)]
    command: CommandEnum,
}

pub fn execute(args: &Command) -> Result<()> {
    match &args.command {
        CommandEnum::File(c) => file::execute(&c),
        CommandEnum::Text(c) => text::execute(&c),
    }
}
