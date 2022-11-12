use std::path::PathBuf;

use clap::{arg, Args};
use human_duration::human_duration;
use rpassword::prompt_password;

use crate::{
    actions::{self, ActionError, ActionResult},
    log,
};

#[derive(Args, Clone)]
/// Encrypt specified file/directory using the passed key
pub struct Command {
    /// Paths of the encrypted file(s)/directory(ies) to
    /// decrypt
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Set output path (only supported when one input path
    /// provided)
    #[arg(short, long)]
    output_path: Option<PathBuf>,

    /// Whether to overwrite the output file/directory
    #[clap(short = 'O', long)]
    overwrite: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn execute(args: &Command) -> ActionResult {
    if args.paths.len() == 0 {
        return Err(ActionError::new("You must provide at least one path", None));
    }

    if args.output_path.is_some() && args.paths.len() != 1 {
        return Err(ActionError::new(
            "Only one input path can be provided when setting an output path",
            None,
        ));
    }

    let key = prompt_password(log::format_info("Enter key: "))
        .map_err(|e| ActionError::new("Failed to read key", Some(format!("{:#?}", e))))?;

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
            None,
        ));
    }

    let (elapsed, success, skipped, failed) = actions::decrypt(
        args.paths.to_owned(),
        args.output_path.to_owned(),
        key,
        args.overwrite,
    )?;

    log::println_success(format!(
        "Decrypted {} files in {}",
        success.len(),
        human_duration(&elapsed)
    ));
    if !failed.is_empty() {
        log::println_error(format!("Failed to decrypt {} files", failed.len()));
    }
    if !skipped.is_empty() {
        log::println_info(format!(
            "{} entr{} were skipped (unknown type)",
            skipped.len(),
            if skipped.len() == 1 { "y" } else { "ies" }
        ));
    }

    Ok(())
}
