use std::{
    fs,
    path::PathBuf,
    sync::mpsc::channel,
    time::{self, Duration},
};

use threadpool::ThreadPool;

use fencryption_lib::{crypto::Crypto, walk_dir::WalkDir};

use crate::{
    actions::{self, decrypt_from_threadpool},
    executions::{ActionError, ActionResult},
};

pub fn decrypt(
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
        .map_err(|e| ActionError::new("Failed to create cipher").error(e))?;

    // Runs for every provided input path.
    for input_path in input_paths {
        let output_path = actions::get_output_dir_path(
            &input_path,
            output_path.as_ref(),
            actions::Command::Decrypt,
        );

        actions::overwrite_when_asked(&input_path, overwrite, actions::Command::Decrypt)?;
        actions::create_output_dir(&output_path, overwrite)?;

        if input_path.is_dir() {
            fs::create_dir(&output_path).ok();

            let walk_dir = WalkDir::new(&input_path)
                .iter()
                .map_err(|e| ActionError::new("Failed to read directory").error(e))?;

            let threadpool = ThreadPool::new(8);
            let (tx, rx) = channel();
            let mut tries_nb = 0;

            for entry in walk_dir {
                let crypto = crypto.clone();

                let entry = entry.map_err(|e| ActionError::new("Failed to read entry").error(e))?;
                let entry_path = entry.path();

                if entry_path.is_file() {
                    tries_nb += 1;
                    let tx = tx.clone();
                    let output_path = output_path.clone();
                    let entry_path = entry_path.clone();
                    threadpool.execute(move || {
                        tx.send((
                            entry_path.to_owned(),
                            match decrypt_from_threadpool(crypto, &entry_path, &output_path, &tx) {
                                Ok(_) => true,
                                Err(_) => false,
                            },
                        ))
                        .unwrap();
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
            let (mut source, mut dest) = actions::create_files(&input_path, &output_path)?;
            match crypto.decrypt_stream(&mut source, &mut dest) {
                Ok(_) => success_paths.push(input_path.to_owned()),
                Err(_) => failed_paths.push(input_path.to_owned()),
            };
        } else {
            skipped_paths.push(input_path.to_owned());
        }

        actions::delete_original_when_asked(
            &input_path,
            delete_original,
            actions::Command::Decrypt,
        )?;
    }

    Ok((
        timer.elapsed().unwrap_or_default(),
        success_paths,
        skipped_paths,
        failed_paths,
    ))
}
