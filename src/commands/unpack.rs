use std::path::PathBuf;

use clap::Args;
use human_duration::human_duration;
use rpassword::prompt_password;

use crate::cli::{
    log,
    util::{CommandError, CommandResult},
};

use super::actions;

#[derive(Args)]
/// Opens a pack
///
/// Creates a directory where the decrypted files appear. To
/// close the pack see command "close".
pub struct Command {
    /// Path of the pack to open
    path: PathBuf,

    /// Permanent unpack
    #[clap(short = 'p', long)]
    permanent: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn execute(args: &Command) -> CommandResult {
    let key = match prompt_password(log::format_info("Enter key: ")) {
        Ok(v) => v,
        Err(e) => {
            return Err(CommandError::new(
                "Failed to read key",
                Some(format!("{:#?}", e)),
            ))
        }
    };

    if key.len() < 1 {
        return Err(CommandError::new(
            "The key cannot be less than 1 character long",
            None,
        ));
    };

    let output_dir_path = PathBuf::from(
        args.path
            .file_stem()
            .ok_or(CommandError::new("Failed to get output path", None))?,
    );

    let elapsed = actions::unpack(
        args.path.to_owned(),
        output_dir_path.to_owned(),
        key.to_owned(),
    )?;

    if args.permanent {
        return Ok(());
    }

    let out = log::prompt(
        "Do you want to update the pack ('u') or exit and discard changes ('q') [u/q] ",
    )
    .map_err(|e| CommandError::new("Failed to read input", Some(format!("{:#?}", e))))?;

    if out == "u" {
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
