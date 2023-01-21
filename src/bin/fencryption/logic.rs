use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use fencryption_lib::{crypto::Crypto, metadata, stream::stream, tmp::TmpFile};
use human_duration::human_duration;
use rpassword::prompt_password;

use crate::{
    error::{Error, ErrorBuilder},
    log, metadata_structs,
    result::Result,
};

pub enum Command {
    Encrypt,
    Decrypt,
    Pack,
    Unpack,
}

pub fn checks<P>(paths: P, output_path: &Option<PathBuf>) -> Result<()>
where
    P: AsRef<Vec<PathBuf>>,
{
    if paths.as_ref().len() == 0 {
        return Err(ErrorBuilder::new()
            .message("Please provide at least one path")
            .build());
    }

    if paths.as_ref().iter().any(|p| !p.exists()) {
        return Err(ErrorBuilder::new()
            .message("I can't work with files that don't exist")
            .build());
    }

    if output_path.as_ref().is_some() && paths.as_ref().len() != 1 {
        return Err(ErrorBuilder::new()
            .debug_message("Only one input path can be provided when setting an output path")
            .build());
    }

    Ok(())
}

pub fn prompt_key(check: bool) -> Result<String> {
    let key = prompt_password(log::format_info("Enter key: ")).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to read key")
            .error(e)
            .build()
    })?;

    if check {
        let confirm_key = prompt_password(log::format_info("Confirm key: ")).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read confirm key")
                .error(e)
                .build()
        })?;

        if key.ne(&confirm_key) {
            return Err(ErrorBuilder::new()
                .message("The two keys don't match")
                .build());
        };

        if key.len() < 1 {
            return Err(ErrorBuilder::new().message("You must set a key").build());
        };

        if key.len() < 6 {
            log::println_warn("Your key should have more than 6 characters to be more secure");
        };
    }

    Ok(key)
}

pub fn change_file_name<P, F>(path: P, callback: F) -> PathBuf
where
    P: AsRef<Path>,
    F: FnOnce(&str) -> String,
{
    let mut path = path.as_ref().to_owned();
    path.set_file_name(
        [callback(
            path.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        )]
        .concat(),
    );
    path
}

pub fn handle_already_existing_item<P>(path: P, overwrite: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    if path.as_ref().exists() {
        if overwrite {
            if path.as_ref().is_dir() {
                fs::remove_dir_all(path.as_ref()).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to overwrite directory, please do it yourself")
                        .error(e)
                        .build()
                })?;
            } else if path.as_ref().is_file() {
                fs::remove_file(path.as_ref()).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to overwrite file, please do it yourself")
                        .error(e)
                        .build()
                })?;
            }
        } else {
            return Err(ErrorBuilder::new()
                .message("The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)")
                .build());
        }
    };
    Ok(())
}

/// Encrypts a file.
///
/// If `with_metadata` is true, writes metadata at the start
/// of the file.
pub fn encrypt_file<P1, P2>(
    crypto: Crypto,
    path: P1,
    output_path: P2,
    with_metadata: bool,
) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let tmp_file = TmpFile::new().map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to create temporary file")
            .error(e)
            .build()
    })?;

    let mut source = OpenOptions::new()
        .read(true)
        .open(path.as_ref())
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read source file")
                .error(e)
                .build()
        })?;

    let mut dest = tmp_file
        .open_with_opts(OpenOptions::new().write(true))
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open temporary file")
                .error(e)
                .build()
        })?;

    if with_metadata {
        dest.write_all(
            &metadata::encode(metadata_structs::FileMetadata::new(&path)).map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to encode file metadata")
                    .error(e)
                    .build()
            })?,
        )
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to write file metadata")
                .error(e)
                .build()
        })?;
    }

    stream(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to transfer data from original to temporary file")
            .error(e)
            .build()
    })?;

    let mut source = tmp_file.open().map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to read temporary file")
            .error(e)
            .build()
    })?;

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path.as_ref())
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open/create destination file")
                .error(e)
                .build()
        })?;

    crypto.encrypt_stream(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to encrypt")
            .error(e)
            .build()
    })?;

    Ok(())
}

/// Decrypts a file.
///
/// If output_path is None, the function will try to extract
/// metadata from the file.
pub fn decrypt_file<P1>(crypto: Crypto, path: P1, output_path: Option<PathBuf>) -> Result<()>
where
    P1: AsRef<Path>,
{
    let tmp_file = TmpFile::new().map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to create temporary file")
            .error(e)
            .build()
    })?;

    let mut source = OpenOptions::new()
        .read(true)
        .open(path.as_ref())
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read source file")
                .error(e)
                .build()
        })?;

    let mut dest = tmp_file
        .open_with_opts(OpenOptions::new().write(true))
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open temporary file")
                .error(e)
                .build()
        })?;

    crypto.decrypt_stream(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to decrypt")
            .error(e)
            .build()
    })?;

    let mut source = tmp_file.open().map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to read temporary file")
            .error(e)
            .build()
    })?;

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(
            output_path.unwrap_or(
                metadata::get_metadata::<metadata_structs::FileMetadata>(&mut source)
                    .map_err(|e| {
                        ErrorBuilder::new()
                            .message("Failed to get file metadata")
                            .error(e)
                            .build()
                    })?
                    .path(),
            ),
        )
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open/create destination file")
                .error(e)
                .build()
        })?;

    stream(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message("Failed to transfer data from temporary to decrypted file")
            .error(e)
            .build()
    })?;

    Ok(())
}

pub fn log_stats(
    success: Vec<PathBuf>,
    failures: Vec<(PathBuf, Error)>,
    skips: Vec<(PathBuf, Error)>,
    elapsed: Duration,
    debug: bool,
    command: Command,
) {
    if !success.is_empty() {
        log::println_success(format!(
            "{} {} file{} in {}",
            match command {
                Command::Encrypt => "Encrypted",
                Command::Decrypt => "Decrypted",
                _ => panic!(),
            },
            success.len(),
            if success.len() == 1 { "" } else { "s" },
            human_duration(&elapsed)
        ));
        if debug {
            success.iter().for_each(|v| {
                log::println_success(log::with_start_line(v.to_str().unwrap(), "  "))
            });
        }
    }
    if !failures.is_empty() {
        log::println_error(format!(
            "Failed to {} {} file{}",
            match command {
                Command::Encrypt => "encrypt",
                Command::Decrypt => "decrypt",
                _ => panic!(),
            },
            failures.len(),
            if failures.len() == 1 { "" } else { "s" }
        ));
        if debug {
            failures.iter().for_each(|v| {
                log::println_error(format!(
                    "{}\n{}",
                    log::with_start_line(v.0.to_str().unwrap_or_default(), "  "),
                    log::with_start_line(v.1.to_string(debug), "      "),
                ));
            });
        }
    }
    if !skips.is_empty() {
        log::println_info(format!(
            "{} entr{} were skipped (unknown type)",
            skips.len(),
            if skips.len() == 1 { "y" } else { "ies" }
        ));
        if debug {
            skips.iter().for_each(|v| {
                log::println_info(format!(
                    "{}\n{}",
                    log::with_start_line(v.0.to_str().unwrap_or_default(), "  "),
                    log::with_start_line(v.1.to_string(debug), "      "),
                ));
            });
        }
    }
}
