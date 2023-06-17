use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use fencryption_lib::{
    crypto::{self, Crypto},
    log, metadata,
};
use human_duration::human_duration;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};

use crate::{
    commands::{Command, CommandOutput},
    error::{ErrorKind, Result},
    text,
    warn::WarnKind,
};

pub fn prompt_key(double_check: bool) -> Result<String> {
    let key = prompt_password(log::format_info(text::PROMPT_KEY)).map_err(ErrorKind::ReadKey)?;

    if double_check {
        let confirm_key = prompt_password(log::format_info(text::PROMPT_CONFIRM_KEY))
            .map_err(ErrorKind::ReadConfirmKey)?;

        if key.ne(&confirm_key) {
            return Err(ErrorKind::KeysNotMatching);
        };

        if key.is_empty() {
            return Err(ErrorKind::NoKey);
        };

        if key.len() < 6 {
            log::println_warn(WarnKind::UnsafeKey.to_string());
        };
    }

    Ok(key)
}

pub fn log_stats(command_output: CommandOutput, debug: bool) {
    match &command_output {
        CommandOutput::EncryptFile {
            success,
            failures,
            skips,
            ..
        }
        | CommandOutput::DecryptFile {
            success,
            failures,
            skips,
            ..
        } => {
            log::println_success(format!(
                "{} {} file{}",
                match command_output {
                    CommandOutput::EncryptFile { .. } => "Encrypted",
                    CommandOutput::DecryptFile { .. } => "Decrypted",
                    _ => unreachable!(),
                },
                success,
                if *success == 1 { "" } else { "s" }
            ));
            if !failures.is_empty() {
                log::println_error(format!(
                    "Failed to {} {} file{}",
                    match command_output {
                        CommandOutput::EncryptFile { .. } => "encrypt",
                        CommandOutput::DecryptFile { .. } => "decrypt",
                        _ => unreachable!(),
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
        _ => {}
    }

    log::println_info(format!(
        "{} elapsed",
        human_duration(&match command_output {
            CommandOutput::EncryptFile { elapsed, .. }
            | CommandOutput::DecryptFile { elapsed, .. }
            | CommandOutput::EncryptText { elapsed, .. }
            | CommandOutput::DecryptText { elapsed, .. } => elapsed,
        })
    ));
}

pub enum OutputDecPath {
    Direct(PathBuf),
    Parent(PathBuf),
}

pub fn checks<P>(input_paths: P, output_path: &Option<PathBuf>) -> Result<()>
where
    P: AsRef<Vec<PathBuf>>,
{
    if input_paths.as_ref().is_empty() {
        return Err(ErrorKind::AtLeastOnePath);
    }

    if input_paths.as_ref().iter().any(|p| !p.exists()) {
        return Err(ErrorKind::FileNotFound);
    }

    if output_path.as_ref().is_some() && input_paths.as_ref().len() != 1 {
        return Err(ErrorKind::OnePathWhenOutputPathSet);
    }

    Ok(())
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

pub fn get_output_paths(
    paths: &[PathBuf],
    output_path: &Option<PathBuf>,
    command: Command,
) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|p| match output_path {
            Some(p) => p.to_owned(),
            None => change_file_name(p, |s| match command {
                Command::EncryptFile => [s, ".enc"].concat(),
                Command::DecryptFile => {
                    if s.ends_with(".enc") {
                        s.replace(".enc", ".dec")
                    } else {
                        [s, ".dec"].concat()
                    }
                } // _ => panic!(),
            }),
        })
        .collect::<Vec<PathBuf>>()
}

pub fn overwrite<P>(paths: P, overwrite: bool) -> Result<()>
where
    P: AsRef<[PathBuf]>,
{
    if paths.as_ref().iter().any(|p| p.exists()) {
        println!("{:#?}, {}", paths.as_ref()[0], paths.as_ref()[0].exists());
        if overwrite {
            for path in paths.as_ref() {
                delete_entry(path).map_err(ErrorKind::Overwrite)?
            }
        } else {
            return Err(ErrorKind::OutputAlreadyExists);
        }
    };

    Ok(())
}

pub fn delete_original<P>(path: P, delete_original: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    if delete_original && path.as_ref().exists() {
        delete_entry(path.as_ref()).map_err(ErrorKind::DeleteOriginal)?;
    }

    Ok(())
}

pub fn delete_entry<P>(path: P) -> Result<(), io::Error>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        fs::remove_dir_all(path.as_ref())?;
    } else if path.as_ref().is_file() {
        fs::remove_file(path.as_ref())?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct PathMetadata(pub PathBuf);

/// Encrypts a file.
pub fn encrypt_file<P1, P2>(
    crypto: Crypto,
    input_path: P1,
    output_path: P2,
    relative_path: Option<PathBuf>,
) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut source = OpenOptions::new()
        .read(true)
        .open(&input_path)
        .map_err(ErrorKind::ReadSource)?;

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&output_path)
        .map_err(ErrorKind::OpenOrCreateDestination)?;

    if let Some(p) = relative_path {
        let metadata = metadata::encode(PathMetadata(p)).map_err(ErrorKind::EncodeMetadata)?;

        let encrypted_metadata = crypto
            .encrypt(metadata)
            .map_err(ErrorKind::EncryptMetadata)?;

        dest.write_all(
            &[
                (encrypted_metadata.len() as u16).to_be_bytes().as_ref(),
                encrypted_metadata.as_ref(),
            ]
            .concat(),
        )
        .map_err(ErrorKind::WriteMetadata)?;
    };

    crypto
        .encrypt_io(&mut source, &mut dest)
        .map_err(|e| match e {
            crypto::ErrorKind::AesError(e) => ErrorKind::AesEncryptFile(e),
            crypto::ErrorKind::Io(e) => ErrorKind::EncryptFileIo(e),
        })?;

    Ok(())
}

/// Decrypts a file.
///
/// If output_path is None, the function will try to extract
/// metadata from the file.
pub fn decrypt_file<P>(crypto: Crypto, input_path: P, output_path: OutputDecPath) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut source = OpenOptions::new()
        .read(true)
        .open(&input_path)
        .map_err(ErrorKind::ReadSource)?;

    let output_path = match output_path {
        OutputDecPath::Direct(p) => p,
        OutputDecPath::Parent(p) => {
            let mut len_bytes = [0u8; 2];
            source
                .read_exact(&mut len_bytes)
                .map_err(ErrorKind::GetEncryptedMetadataLength)?;
            let len = u16::from_be_bytes(len_bytes) as usize;
            let mut metadata_bytes = vec![0u8; len];
            source
                .read_exact(&mut metadata_bytes)
                .map_err(ErrorKind::GetEncryptedMetadata)?;
            let metadata = metadata::decode::<PathMetadata>(
                &crypto
                    .decrypt(&metadata_bytes)
                    .map_err(ErrorKind::DecryptMetadata)?,
            )
            .map_err(ErrorKind::DecodeMetadata)?;

            let path = p.join(metadata.0);
            if let Some(p) = path.parent() {
                fs::create_dir_all(p).map_err(ErrorKind::CreateSubDir)?
            };
            path
        }
    };

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .map_err(ErrorKind::OpenOrCreateDestination)?;

    crypto
        .decrypt_io(&mut source, &mut dest)
        .map_err(|e| match e {
            crypto::ErrorKind::AesError(e) => ErrorKind::AesDecryptFile(e),
            crypto::ErrorKind::Io(e) => ErrorKind::DecryptFileIo(e),
        })?;

    Ok(())
}
