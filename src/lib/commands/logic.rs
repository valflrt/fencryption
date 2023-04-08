//! Logic used in commands.

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    commands::{Command, ErrorBuilder, Result},
    crypto::{self, Crypto},
    metadata,
};

pub enum OutputDecPath {
    Direct(PathBuf),
    Parent(PathBuf),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PathMetadata(PathBuf);

impl PathMetadata {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        PathMetadata(path.as_ref().to_owned())
    }

    pub fn path(&self) -> PathBuf {
        self.0.to_owned()
    }
}

pub fn checks<P>(input_paths: P, output_path: &Option<PathBuf>) -> Result<()>
where
    P: AsRef<Vec<PathBuf>>,
{
    if input_paths.as_ref().len() == 0 {
        return Err(ErrorBuilder::new()
            .message("Please provide at least one path")
            .build());
    }

    if input_paths.as_ref().iter().any(|p| !p.exists()) {
        return Err(ErrorBuilder::new()
            .message("I can't work with files that don't exist")
            .build());
    }

    if output_path.as_ref().is_some() && input_paths.as_ref().len() != 1 {
        return Err(ErrorBuilder::new()
            .message("Only one input path can be provided when setting an output path")
            .build());
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
    paths: &Vec<PathBuf>,
    output_path: &Option<PathBuf>,
    command: Command,
) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|p| match output_path {
            Some(p) => p.to_owned(),
            None => change_file_name(p, |s| match command {
                Command::Encrypt => [s, ".enc"].concat(),
                Command::Decrypt => {
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
                delete_entry(&path).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to overwrite file/directory, please do it yourself")
                        .error(e)
                        .build()
                })?
            }
        } else {
            return Err(ErrorBuilder::new()
                .message("The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)")
                .build());
        }
    };

    Ok(())
}

pub fn delete_original<P>(path: P, delete_original: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    if delete_original && path.as_ref().exists() {
        delete_entry(path.as_ref()).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to delete original file/directory, please do it yourself")
                .error(e)
                .build()
        })?;
    }

    Ok(())
}

pub fn delete_entry<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        fs::remove_dir_all(path.as_ref()).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to remove directory, please do it yourself")
                .error(e)
                .build()
        })?;
    } else if path.as_ref().is_file() {
        fs::remove_file(path.as_ref()).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to remove file, please do it yourself")
                .error(e)
                .build()
        })?;
    }

    Ok(())
}

/// Encrypts a file.
///
/// If `with_metadata` is true, writes metadata at the start
/// of the file.
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
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read source file")
                .error(e)
                .build()
        })?;

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&output_path)
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open/create destination file")
                .error(e)
                .build()
        })?;

    if let Some(p) = relative_path {
        let metadata = metadata::encode(PathMetadata::new(p)).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to encode file metadata")
                .error(e)
                .build()
        })?;

        let encrypted_metadata = crypto.encrypt(metadata).map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to encrypt metadata")
                .error(e)
                .build()
        })?;

        dest.write_all(
            &[
                (encrypted_metadata.len() as u16).to_be_bytes().as_ref(),
                encrypted_metadata.as_ref(),
            ]
            .concat(),
        )
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to write metadata")
                .error(e)
                .build()
        })?;
    };

    crypto.encrypt_io(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message(match e {
                crypto::ErrorKind::AesError(_) => "Failed to encrypt file (key must be wrong)",
                _ => "Failed to encrypt file",
            })
            .error(e)
            .build()
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
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to read source file")
                .error(e)
                .build()
        })?;

    let output_path = match output_path {
        OutputDecPath::Direct(p) => p,
        OutputDecPath::Parent(p) => {
            let mut len_bytes = [0u8; 2];
            source.read_exact(&mut len_bytes).map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to get encrypted metadata length")
                    .error(e)
                    .build()
            })?;
            let len = u16::from_be_bytes(len_bytes) as usize;
            let mut metadata_bytes = vec![0u8; len];
            source.read_exact(&mut metadata_bytes).map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to get encrypted metadata")
                    .error(e)
                    .build()
            })?;
            let metadata = metadata::decode::<PathMetadata>(
                &crypto.decrypt(&metadata_bytes).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to decrypt metadata")
                        .error(e)
                        .build()
                })?,
            )
            .map_err(|e| {
                ErrorBuilder::new()
                    .message("Failed to decode metadata")
                    .error(e)
                    .build()
            })?;

            let path = p.join(metadata.path());
            if let Some(p) = path.parent() {
                fs::create_dir_all(p).map_err(|e| {
                    ErrorBuilder::new()
                        .message("Failed to create sub-directory")
                        .error(e)
                        .build()
                })?
            };
            path
        }
    };

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&output_path)
        .map_err(|e| {
            ErrorBuilder::new()
                .message("Failed to open/create destination file")
                .error(e)
                .build()
        })?;

    crypto.decrypt_io(&mut source, &mut dest).map_err(|e| {
        ErrorBuilder::new()
            .message(match e {
                crypto::ErrorKind::AesError(_) => "Failed to decrypt file (key must be wrong)",
                _ => "Failed to decrypt file",
            })
            .error(e)
            .build()
    })?;

    Ok(())
}
