use std::{
    fs,
    iter::zip,
    path::PathBuf,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose, Engine};
use fencryption_lib::{crypto::Crypto, log, walk_dir::walk_dir};
use threadpool::ThreadPool;
use uuid::Uuid;

use crate::{
    error::{ErrorKind, Result},
    text,
    util::{self},
};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    EncryptFile,
    DecryptFile,
}

#[derive(Debug)]
pub enum CommandOutput {
    EncryptFile {
        success: usize,
        failures: Vec<(PathBuf, ErrorKind)>,
        skips: Vec<(PathBuf, ErrorKind)>,
        elapsed: Duration,
    },
    DecryptFile {
        success: usize,
        failures: Vec<(PathBuf, ErrorKind)>,
        skips: Vec<(PathBuf, ErrorKind)>,
        elapsed: Duration,
    },
    EncryptText {
        elapsed: Duration,
    },
    DecryptText {
        elapsed: Duration,
    },
}

pub fn encrypt_file(
    key: String,
    paths: &Vec<PathBuf>,
    output_path: &Option<PathBuf>,
    overwrite: bool,
    delete_original: bool,
) -> Result<CommandOutput> {
    log::println_info(text::START_ENCRYPTION);

    let start_time = Instant::now();

    let output_paths = util::get_output_paths(paths, output_path, Command::EncryptFile);

    util::checks(paths, output_path)?;
    util::overwrite(&output_paths, overwrite)?;

    let mut success: usize = 0;
    let mut failures: Vec<(PathBuf, ErrorKind)> = Vec::new();
    let mut skips: Vec<(PathBuf, ErrorKind)> = Vec::new();

    let crypto = Crypto::new(key);

    for (main_input_path, main_output_path) in zip(paths, output_paths) {
        match main_input_path.to_owned() {
            dir_path if dir_path.is_dir() => {
                fs::create_dir(&main_output_path).map_err(ErrorKind::CreateOutputDir)?;

                let walk_dir = walk_dir(&dir_path).map_err(ErrorKind::ReadDir)?;

                let threadpool = ThreadPool::new(8);
                let (tx, rx) = channel();
                let mut tries_n = 0;

                for dir_entry in walk_dir {
                    let entry = dir_entry.map_err(ErrorKind::ReadDirEntry)?;
                    let input_path = entry.path();
                    let relative_entry_path = input_path
                        .strip_prefix(&dir_path)
                        .map_err(ErrorKind::GetRelativePath)?
                        .to_owned();
                    let output_path = main_output_path.join(Uuid::new_v4().to_string());

                    match input_path {
                        input_path if input_path.is_file() => {
                            tries_n += 1;

                            let crypto = crypto.clone();
                            let tx = tx.clone();
                            let output_path = output_path.clone();

                            threadpool.execute(move || {
                                let result = util::encrypt_file(
                                    &crypto,
                                    &input_path,
                                    &output_path,
                                    Some(relative_entry_path),
                                );
                                tx.send((input_path, result)).unwrap();
                            });
                        }
                        input_path => {
                            if !input_path.is_dir() {
                                skips.push((input_path, ErrorKind::UnknownFileType));
                            }
                        }
                    }
                }

                threadpool.join();
                rx.iter()
                    .take(tries_n)
                    .for_each(|(path, result)| match result {
                        Ok(_) => success += 1,
                        Err(e) => failures.push((path, e)),
                    })
            }
            path if path.is_file() => {
                match util::encrypt_file(&crypto, &path, &main_output_path, None) {
                    Ok(_) => success += 1,
                    Err(e) => failures.push((path, e)),
                };
            }
            path => skips.push((path, ErrorKind::UnknownFileType)),
        }

        util::delete_original(&main_input_path, delete_original)?;
    }

    Ok(CommandOutput::EncryptFile {
        success,
        failures,
        skips,
        elapsed: start_time.elapsed(),
    })
}

pub fn decrypt_file(
    key: String,
    paths: &Vec<PathBuf>,
    output_path: &Option<PathBuf>,
    overwrite: bool,
    delete_original: bool,
) -> Result<CommandOutput> {
    log::println_info(text::START_DECRYPTION);

    let start_time = Instant::now();

    let output_paths = util::get_output_paths(paths, output_path, Command::DecryptFile);

    util::checks(paths, output_path)?;
    util::overwrite(&output_paths, overwrite)?;

    let mut success: usize = 0;
    let mut failures: Vec<(PathBuf, ErrorKind)> = Vec::new();
    let mut skips: Vec<(PathBuf, ErrorKind)> = Vec::new();

    let crypto = Crypto::new(key);

    for (main_input_path, main_output_path) in zip(paths, output_paths) {
        match main_input_path.to_owned() {
            dir_path if dir_path.is_dir() => {
                fs::create_dir(&main_output_path).map_err(ErrorKind::CreateOutputDir)?;

                let walk_dir = walk_dir(&dir_path).map_err(ErrorKind::ReadDir)?;

                let threadpool = ThreadPool::new(8);
                let (tx, rx) = channel();
                let mut tries_n = 0;

                for dir_entry in walk_dir {
                    let entry = dir_entry.map_err(ErrorKind::ReadDirEntry)?;
                    let input_path = entry.path();

                    match input_path {
                        input_path if input_path.is_file() => {
                            tries_n += 1;

                            let crypto = crypto.clone();
                            let tx = tx.clone();
                            let output_dir_path = main_output_path.clone();

                            threadpool.execute(move || {
                                let result = util::decrypt_file(
                                    &crypto,
                                    &input_path,
                                    util::OutputPath::Parent(output_dir_path),
                                );
                                tx.send((input_path, result)).unwrap();
                            });
                        }
                        input_path => {
                            if !input_path.is_dir() {
                                skips.push((input_path, ErrorKind::UnknownFileType));
                            }
                        }
                    }
                }

                threadpool.join();
                rx.iter()
                    .take(tries_n)
                    .for_each(|(path, result)| match result {
                        Ok(_) => success += 1,
                        Err(e) => failures.push((path, e)),
                    })
            }
            path if path.is_file() => {
                match util::decrypt_file(&crypto, &path, util::OutputPath::Exact(main_output_path))
                {
                    Ok(_) => success += 1,
                    Err(e) => failures.push((path, e)),
                };
            }
            path => skips.push((path, ErrorKind::UnknownFileType)),
        }

        util::delete_original(&main_input_path, delete_original)?;
    }

    Ok(CommandOutput::DecryptFile {
        success,
        failures,
        skips,
        elapsed: start_time.elapsed(),
    })
}

pub fn encrypt_text(key: String, text: &str) -> Result<CommandOutput> {
    let start_time = Instant::now();

    let crypto = Crypto::new(key);

    let enc = crypto
        .encrypt(text.as_bytes())
        .map_err(ErrorKind::EncryptText)?;

    log::println_success(format!(
        "Successfully encrypted text: base64 {}",
        general_purpose::STANDARD.encode(enc)
    ));

    Ok(CommandOutput::EncryptText {
        elapsed: start_time.elapsed(),
    })
}

pub fn decrypt_text(key: String, encrypted: &str) -> Result<CommandOutput> {
    let start_time = Instant::now();

    let crypto = Crypto::new(key);

    let enc = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(ErrorKind::DecodeBase64)?;

    let dec = crypto.decrypt(enc).map_err(ErrorKind::DecryptText)?;

    log::println_success(format!(
        "Successfully decrypted text: \"{}\"",
        String::from_utf8(dec).map_err(ErrorKind::ConvertUtf8)?
    ));

    Ok(CommandOutput::DecryptText {
        elapsed: start_time.elapsed(),
    })
}
