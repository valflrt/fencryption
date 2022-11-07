use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    time,
};

use clap::Args;
use human_duration::human_duration;
use rpassword::{self, prompt_password};

use crate::cli::{
    log,
    util::{ActionError, ActionResult},
};
use fencryption::{crypto::Crypto, pack::Pack, tmp_dir::TmpDir};

#[derive(Args)]
/// Packs a directory: creates a file in which the specified
/// directory is packed. The original directory is deleted.
pub struct Command {
    /// Path of the directory to pack
    path: PathBuf,

    /// Overwrite the output pack
    #[clap(short = 'O', long)]
    overwrite: bool,

    #[clap(from_global)]
    debug: bool,
}

pub fn action(args: &Command) -> ActionResult {
    if !args.path.is_dir() {
        return Err(ActionError::new("The path must lead to a directory", None));
    };

    let key = match prompt_password(log::format_info("Enter key: ")) {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read key",
                Some(format!("  - {:?}", e)),
            ))
        }
    };
    let confirm_key = match prompt_password(log::format_info("Confirm key: ")) {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read confirm key",
                Some(format!("  - {:?}", e)),
            ))
        }
    };

    if key.ne(&confirm_key) {
        return Err(ActionError::new("The two keys don't match", None));
    };

    if key.len() < 1 {
        return Err(ActionError::new(
            "The key cannot be less than 1 character long",
            None,
        ));
    };

    let timer = time::SystemTime::now();

    let crypto = match Crypto::new(&key.as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to create cipher",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    let tmp_dir = match TmpDir::new() {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to create temporary directory",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    let tmp_pack_path = tmp_dir.gen_path();
    let mut pack_path = args.path.clone();
    pack_path.set_file_name(
        [
            pack_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
            ".pack",
        ]
        .concat(),
    );

    match Pack::new(&tmp_pack_path).create(&args.path) {
        Ok(_) => {
            log::println_success("Created pack");
        }
        Err(e) => {
            return Err(ActionError::new(
                "Failed to create pack",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    if let Err(e) = fs::remove_dir_all(&args.path) {
        return Err(ActionError::new(
            "Failed to remove original directory",
            if args.debug == true {
                Some(format!("  - {:?}", e))
            } else {
                None
            },
        ));
    };

    let mut source = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(&tmp_pack_path)
    {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read pack file",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };
    let mut dest = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&pack_path)
    {
        Ok(v) => v,
        Err(e) => {
            return Err(ActionError::new(
                "Failed to read/create destination file",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    match crypto.encrypt_stream(&mut source, &mut dest) {
        Ok(_) => {
            log::println_success("Encrypted pack");
        }
        Err(e) => {
            return Err(ActionError::new(
                "Failed to encrypt pack",
                if args.debug == true {
                    Some(format!("  - {:?}", e))
                } else {
                    None
                },
            ));
        }
    };

    Ok(Some(format!(
        "{} elapsed",
        human_duration(&timer.elapsed().unwrap_or_default())
    )))
}
