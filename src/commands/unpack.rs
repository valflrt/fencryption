use std::{fs, path::PathBuf};

use clap::Args;
use human_duration::human_duration;
use rpassword::prompt_password;

use crate::{
    actions::{self, ActionError, ActionResult},
    log,
};

#[derive(Args)]
/// Open a pack
///
/// Creates a directory where the decrypted files appear. To
/// close the pack see command "close".
pub struct Command {
    /// Path of the pack to open
    #[clap(required = true)]
    path: PathBuf,

    /// Permanent unpack
    #[clap(short = 'p', long)]
    permanent: bool,

    /// Overwrite if the directory already exists
    #[clap(short = 'O', long)]
    overwrite: bool,

    #[clap(from_global)]
    pub debug: bool,
}

pub fn execute(args: &Command) -> ActionResult {
    let key = match prompt_password(log::format_info("Enter key: ")) {
        Ok(v) => v,
        Err(e) => return Err(ActionError::new_with_error("Failed to read key", e)),
    };

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
        ));
    }

    let output_dir_path = PathBuf::from(
        args.path
            .file_stem()
            .ok_or(ActionError::new("Failed to get output path"))?,
    );

    if output_dir_path.exists() {
        if args.overwrite {
            if output_dir_path.is_dir() {
                fs::remove_dir_all(&output_dir_path).map_err(|e| {
                    ActionError::new_with_error("Failed to overwrite output directory", e)
                })?;
            } else if output_dir_path.is_file() {
                fs::remove_file(&output_dir_path).map_err(|e| {
                    ActionError::new_with_error("Failed to overwrite output file", e)
                })?;
            }
        } else {
            return Err(ActionError::new("The output directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)"));
        }
    }

    log::println_info("Unpacking...");

    let elapsed = actions::unpack(
        args.path.to_owned(),
        output_dir_path.to_owned(),
        key.to_owned(),
    )?;

    if args.permanent {
        return Ok(());
    }

    let out = log::prompt(
        "Do you want to update the pack ('u') or exit and discard changes ('q') [u/Q] ",
    )
    .map_err(|e| ActionError::new_with_error("Failed to read input", e))?;
    let out = out.trim();

    if out == "u" {
        log::println_info("Updating pack...");
        let elapsed = actions::pack(output_dir_path, key, true)?;
        log::println_success(format!(
            "Updated pack ({} elapsed)",
            human_duration(&elapsed)
        ));
    } else {
        log::println_info(format!(
            "Exited without saving changes ({} elapsed)",
            human_duration(&elapsed)
        ));
    }

    Ok(())
}
