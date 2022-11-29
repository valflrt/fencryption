use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::mpsc::channel,
    time::{self, Duration},
};

use threadpool::ThreadPool;

use fencryption_lib::{crypto::Crypto, stream::stream, tmp::TmpFile, walk_dir::WalkDir};
use uuid::Uuid;

use crate::actions::{ActionError, ActionResult};

pub fn encrypt(
    input_paths: Vec<PathBuf>,
    output_path: Option<PathBuf>,
    key: String,
    overwrite: bool,
    delete_original: bool,
) -> ActionResult<(Duration, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let mut success_paths: Vec<PathBuf> = Vec::new();
    let mut skipped_paths: Vec<PathBuf> = Vec::new();
    let mut failed_paths: Vec<PathBuf> = Vec::new();
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| ActionError::new_with_error("Failed to create cipher", e))?;

    // Runs for every provided input path.
    for input_path in input_paths {
        let output_path = match &output_path {
            Some(v) => v.to_owned(),
            None => {
                let mut path = input_path.to_owned();
                path.set_file_name(
                    [
                        path.file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                        ".enc",
                    ]
                    .concat(),
                );
                path
            }
        };

        if output_path.exists() {
            if overwrite {
                if output_path.is_dir() {
                    fs::remove_dir_all(&output_path).map_err(|e| {
                        ActionError::new_with_error("Failed to overwrite output directory", e)
                    })?;
                } else if output_path.is_file() {
                    fs::remove_file(&output_path).map_err(|e| {
                        ActionError::new_with_error("Failed to overwrite output file", e)
                    })?;
                }
            } else {
                return Err(ActionError::new(
                    "The output file/directory already exists (use \"--overwrite\"/\"-O\" to force overwrite)",
                ));
            }
        }

        if input_path.is_dir() {
            fs::create_dir(&output_path).ok();

            let walk_dir = WalkDir::new(&input_path)
                .iter()
                .map_err(|e| ActionError::new_with_error("Failed to read directory", e))?;

            let threadpool = ThreadPool::new(8);
            let (tx, rx) = channel();
            let mut tries_nb = 0;

            for entry in walk_dir {
                let crypto = crypto.clone();

                let entry =
                    entry.map_err(|e| ActionError::new_with_error("Failed to read entry", e))?;
                let entry_path = entry.path();
                let new_entry_path = output_path.join(Uuid::new_v4().to_string());

                if entry_path.is_file() {
                    tries_nb += 1;
                    let tx = tx.clone();
                    let input_path = input_path.clone();
                    let entry_path = entry_path.clone();
                    threadpool.execute(move || {
                        let tmp_file = match TmpFile::new() {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };

                        let mut source = match OpenOptions::new().read(true).open(&entry_path) {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };
                        let mut dest = match tmp_file.open_with_opts(OpenOptions::new().write(true))
                        {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };

                        let entry_path = match entry_path.strip_prefix(input_path) {
                            Ok(v) => v.to_path_buf(),
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };

                        let path_as_bytes = match entry_path.to_str() {
                            Some(v) => v,
                            None => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        }
                        .as_bytes();

                        if let Err(_) = dest.write_all(&(path_as_bytes.len() as u32).to_be_bytes())
                        {
                            tx.send((entry_path, false)).unwrap();
                            return;
                        };

                        if let Err(_) = dest.write_all(path_as_bytes) {
                            tx.send((entry_path, false)).unwrap();
                            return;
                        };

                        if let Err(_) = stream(&mut source, &mut dest) {
                            tx.send((entry_path, false)).unwrap();
                            return;
                        }

                        let mut source = match tmp_file.open() {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };
                        let mut dest = match OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(&new_entry_path)
                        {
                            Ok(v) => v,
                            Err(_) => {
                                tx.send((entry_path, false)).unwrap();
                                return;
                            }
                        };

                        if let Err(_) = crypto.encrypt_stream(&mut source, &mut dest) {
                            tx.send((entry_path, false)).unwrap();
                            return;
                        }

                        tx.send((entry_path, true)).unwrap();
                    });
                } else if !entry_path.is_dir() {
                    skipped_paths.push(entry_path.to_owned());
                }
            }

            threadpool.join();
            rx.iter().take(tries_nb).for_each(|(path, success)| {
                if success {
                    success_paths.push(path);
                } else {
                    failed_paths.push(path);
                }
            })
        } else if input_path.is_file() {
            let mut source = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&input_path)
                .map_err(|e| ActionError::new_with_error("Failed to read source file", e))?;
            let mut dest = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&output_path)
                .map_err(|e| {
                    ActionError::new_with_error("Failed to read/create destination file", e)
                })?;

            match crypto.encrypt_stream(&mut source, &mut dest) {
                Ok(_) => success_paths.push(input_path.to_owned()),
                Err(_) => failed_paths.push(input_path.to_owned()),
            };
        } else {
            skipped_paths.push(input_path.to_owned());
        }

        if delete_original && input_path.exists() {
            if input_path.is_dir() {
                fs::remove_dir_all(input_path).map_err(|e| {
                    ActionError::new_with_error("Failed to remove original directory", e)
                })?;
            } else if input_path.is_file() {
                fs::remove_file(input_path).map_err(|e| {
                    ActionError::new_with_error("Failed to remove original file", e)
                })?;
            } else {
                return Err(ActionError::new(
                    "Failed to remove original item (unknown type)",
                ));
            }
        }
    }

    Ok((
        timer.elapsed().unwrap_or_default(),
        success_paths,
        skipped_paths,
        failed_paths,
    ))
}
