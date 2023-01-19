use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use fencryption_lib::{crypto::Crypto, stream::stream, tmp::TmpFile};

use crate::executions::{ActionError, ActionResult};

pub enum Command {
    Encrypt,
    Decrypt,
    Pack,
}

pub fn get_output_dir_path<P1, P2>(
    input_path: P1,
    output_path: Option<P2>,
    command: Command,
) -> PathBuf
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    match output_path.as_ref() {
        Some(v) => v.as_ref().to_owned(),
        None => {
            let mut path = input_path.as_ref().to_owned();
            path.set_file_name(
                [
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                    match command {
                        Command::Encrypt => ".enc",
                        Command::Decrypt => ".dec",
                        _ => "",
                    },
                ]
                .concat(),
            );
            path
        }
    }
}

pub fn get_pack_path<P>(input_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut path = input_path.as_ref().to_owned();
    path.set_file_name(
        [
            input_path
                .as_ref()
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
            ".pack",
        ]
        .concat(),
    );
    path
}

pub fn create_output_dir<P>(output_path: P, overwrite: bool) -> ActionResult
where
    P: AsRef<Path>,
{
    if output_path.as_ref().exists() {
        if overwrite {
            if output_path.as_ref().is_dir() {
                fs::remove_dir_all(output_path.as_ref()).map_err(|e| {
                    ActionError::new("Failed to overwrite output directory").error(e)
                })?;
            } else if output_path.as_ref().is_file() {
                fs::remove_file(output_path.as_ref())
                    .map_err(|e| ActionError::new("Failed to overwrite output file").error(e))?;
            }
        } else {
            return Err(ActionError::new(
                "The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)"
            ));
        }
    }

    Ok(())
}

pub fn create_files<P1, P2>(input_path: P1, output_path: P2) -> ActionResult<(File, File)>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    Ok((
        OpenOptions::new()
            .read(true)
            .open(&input_path)
            .map_err(|e| ActionError::new("Failed to read source file").error(e))?,
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&output_path)
            .map_err(|e| ActionError::new("Failed to read/create destination file").error(e))?,
    ))
}

pub fn delete_original_when_asked<P>(
    input_path: P,
    delete_original: bool,
    command: Command,
) -> ActionResult
where
    P: AsRef<Path>,
{
    match command {
        Command::Encrypt | Command::Decrypt => {
            if delete_original && input_path.as_ref().exists() {
                if input_path.as_ref().is_dir() {
                    fs::remove_dir_all(input_path).map_err(|e| {
                        ActionError::new("Failed to remove original directory").error(e)
                    })?;
                } else if input_path.as_ref().is_file() {
                    fs::remove_file(input_path)
                        .map_err(|e| ActionError::new("Failed to remove original file").error(e))?;
                } else {
                    return Err(ActionError::new(
                        "Failed to remove original item (unknown type)",
                    ));
                }
            }
        }
        Command::Pack => {
            if delete_original && input_path.as_ref().exists() {
                fs::remove_dir_all(&input_path).map_err(|e| {
                    ActionError::new("Failed to remove original directory").error(e)
                })?;
            }
        }
    }

    Ok(())
}

pub fn overwrite_when_asked<P>(input_path: P, overwrite: bool, command: Command) -> ActionResult
where
    P: AsRef<Path>,
{
    match command {
        Command::Encrypt | Command::Decrypt => {
            if overwrite && input_path.as_ref().exists() {
                if input_path.as_ref().is_dir() {
                    fs::remove_dir_all(input_path).map_err(|e| {
                        ActionError::new("Failed to remove original directory").error(e)
                    })?;
                } else if input_path.as_ref().is_file() {
                    fs::remove_file(input_path)
                        .map_err(|e| ActionError::new("Failed to remove original file").error(e))?;
                } else {
                    return Err(ActionError::new(
                        "Failed to remove original item (unknown type)",
                    ));
                }
            }
        }
        Command::Pack => {
            if overwrite {
                fs::remove_dir_all(&input_path).map_err(|e| {
                    ActionError::new("Failed to remove original directory").error(e)
                })?;
            }
        }
    }

    Ok(())
}

pub fn encrypt_from_threadpool<P1, P2, P3>(
    crypto: Crypto,
    entry_path: P1,
    new_entry_path: P2,
    input_path: P3,
    tx: &Sender<(PathBuf, bool)>,
) -> ActionResult
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    P3: AsRef<Path>,
{
    let tmp_file =
        TmpFile::new().map_err(|e| ActionError::new("Failed to create temporary file").error(e))?;

    let mut source = OpenOptions::new()
        .read(true)
        .open(&entry_path)
        .map_err(|e| ActionError::new("Failed to read source file").error(e))?;
    let mut dest = tmp_file
        .open_with_opts(OpenOptions::new().write(true))
        .map_err(|e| ActionError::new("Failed to read/create destination file").error(e))?;

    let entry_path = entry_path
        .as_ref()
        .strip_prefix(input_path)
        .map_err(|e| ActionError::new("Failed to get relative entry path").error(e))?;

    let path_as_bytes = entry_path
        .to_str()
        .ok_or(ActionError::new("Failed to read path as bytes"))?
        .as_bytes();

    dest.write_all(&(path_as_bytes.len() as u32).to_be_bytes())
        .map_err(|e| ActionError::new("Failed to write to destination file").error(e))?;

    dest.write_all(path_as_bytes)
        .map_err(|e| ActionError::new("Failed to write to destination file").error(e))?;

    stream(&mut source, &mut dest).map_err(|e| {
        ActionError::new("Failed to transfer data from temporary file to destination file").error(e)
    })?;

    let mut source = tmp_file
        .open()
        .map_err(|e| ActionError::new("Failed to read source file").error(e))?;
    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&new_entry_path)
        .map_err(|e| ActionError::new("Failed to read/create destination file").error(e))?;

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .map_err(|e| ActionError::new("Failed to encrypt").error(e))?;

    tx.send((entry_path.to_owned(), true))
        .map_err(|e| ActionError::new("Failed to report entry status").error(e))?;

    Ok(())
}

pub fn decrypt_from_threadpool<P1, P2>(
    crypto: Crypto,
    entry_path: P1,
    output_path: P2,
    tx: &Sender<(PathBuf, bool)>,
) -> ActionResult
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let tmp_file =
        TmpFile::new().map_err(|e| ActionError::new("Failed to create temporary file").error(e))?;

    let mut source = OpenOptions::new()
        .read(true)
        .open(&entry_path)
        .map_err(|e| ActionError::new("Failed to read source file").error(e))?;
    let mut dest = tmp_file
        .open_with_opts(OpenOptions::new().write(true))
        .map_err(|e| ActionError::new("Failed to read/create destination file").error(e))?;

    crypto
        .decrypt_stream(&mut source, &mut dest)
        .map_err(|e| ActionError::new("Failed to decrypt").error(e))?;

    let mut source = tmp_file
        .open()
        .map_err(|e| ActionError::new("Failed to read source file").error(e))?;

    let mut path_bytes_len = [0u8; 4];
    source
        .read_exact(&mut path_bytes_len)
        .map_err(|e| ActionError::new("Failed to read destination file").error(e))?;
    let path_bytes_len = u32::from_be_bytes(path_bytes_len) as usize;

    let mut path_bytes = vec![0u8; path_bytes_len];
    source
        .read_exact(&mut path_bytes)
        .map_err(|e| ActionError::new("Failed to read destination file").error(e))?;
    let path = std::str::from_utf8(&path_bytes)
        .map_err(|e| ActionError::new("Failed output entry path").error(e))?;
    let new_entry_path = output_path.as_ref().join(path);

    if let Some(v) = new_entry_path.parent() {
        fs::create_dir_all(v).ok();
    }

    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&new_entry_path)
        .map_err(|e| ActionError::new("Failed to read/create destination file").error(e))?;

    stream(&mut source, &mut dest).map_err(|e| {
        ActionError::new("Failed to transfer data from temporary file to destination file").error(e)
    })?;

    tx.send((entry_path.as_ref().to_owned(), true)).unwrap();

    Ok(())
}
