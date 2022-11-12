use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
    time::{self, Duration},
};

use fencryption_lib::{crypto::Crypto, pack::Pack, tmp_dir::TmpDir};

use crate::cli::{CommandError, CommandResult};

pub fn pack(input_path: PathBuf, key: String, delete_original: bool) -> CommandResult<Duration> {
    if !input_path.is_dir() {
        return Err(CommandError::new("The path must lead to a directory", None));
    };

    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| CommandError::new("Failed to create cipher", Some(format!("{:#?}", e))))?;

    let tmp_dir = TmpDir::new().map_err(|e| {
        CommandError::new(
            "Failed to create temporary directory",
            Some(format!("{:#?}", e)),
        )
    })?;

    let tmp_pack_path = tmp_dir.gen_path();
    let mut pack_path = input_path.to_path_buf();
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

    Pack::new(&tmp_pack_path)
        .create(&input_path)
        .map_err(|e| CommandError::new("Failed to create pack", Some(format!("{:#?}", e))))?;

    if delete_original {
        fs::remove_dir_all(&input_path).map_err(|e| {
            CommandError::new(
                "Failed to remove original directory",
                Some(format!("{:#?}", e)),
            )
        })?;
    }

    let mut source = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&tmp_pack_path)
        .map_err(|e| CommandError::new("Failed to read pack file", Some(format!("{:#?}", e))))?;
    let mut dest = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&pack_path)
        .map_err(|e| {
            CommandError::new(
                "Failed to read/create destination file",
                Some(format!("{:#?}", e)),
            )
        })?;

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .map_err(|e| CommandError::new("Failed to encrypt pack", Some(format!("{:#?}", e))))?;

    Ok(timer.elapsed().unwrap_or_default())
}
