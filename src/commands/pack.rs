use std::path::PathBuf;

use clap::Args;
use human_duration::human_duration;
use rpassword::{self, prompt_password};

use crate::{
    actions::{self, ActionError, ActionResult},
    log,
};

#[derive(Args)]
/// Pack a directory
///
/// Creates a file in which the specified directory is packed.
/// The original directory is deleted.
pub struct Command {
    /// Path of the directory to pack
    #[clap(required = true)]
    path: PathBuf,

    /// Overwrite the output pack
    #[clap(short = 'O', long)]
    overwrite: bool,

    /// Delete the original directory after creating pack
    #[clap(short = 'd', long)]
    delete_original: bool,

    #[clap(from_global)]
    pub debug: bool,
}

pub fn execute(args: &Command) -> ActionResult {
    let key = prompt_password(log::format_info("Enter key: "))
        .map_err(|e| ActionError::new_with_error("Failed to read key", e))?;
    let confirm_key = prompt_password(log::format_info("Confirm key: "))
        .map_err(|e| ActionError::new_with_error("Failed to read confirm key", e))?;

    if key.ne(&confirm_key) {
        return Err(ActionError::new("The two keys don't match"));
    }

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
        ));
    }

    log::println_info("Packing...");

    let elapsed = actions::pack(args.path.to_owned(), key, args.delete_original)?;

    log::println_success(format!("{} elapsed", human_duration(&elapsed)));

    Ok(())
}
