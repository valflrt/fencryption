use std::{
    fs::OpenOptions,
    path::PathBuf,
    time::{self, Duration},
};

use fencryption_lib::{crypto::Crypto, pack::Pack, tmp_dir::TmpDir};

use crate::actions::{ActionError, ActionResult};

pub fn unpack(
    input_path: PathBuf,
    output_dir_path: PathBuf,
    key: String,
) -> ActionResult<Duration> {
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| ActionError::new("Failed to create cipher", Some(format!("{:#?}", e))))?;

    let tmp_dir = TmpDir::new().map_err(|e| {
        ActionError::new(
            "Failed to create temporary directory",
            Some(format!("{:#?}", e)),
        )
    })?;

    let tmp_pack_path = tmp_dir.unique_path();

    let mut source = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&input_path)
        .map_err(|e| ActionError::new("Failed to read pack file", Some(format!("{:#?}", e))))?;
    let mut dest = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&tmp_pack_path)
        .map_err(|e| {
            ActionError::new(
                "Failed to read/create temporary decrypted pack file",
                Some(format!("{:#?}", e)),
            )
        })?;

    crypto
        .decrypt_stream(&mut source, &mut dest)
        .map_err(|e| ActionError::new("Failed to decrypt pack", Some(format!("{:#?}", e))))?;

    Pack::new(&tmp_pack_path)
        .unpack(&output_dir_path)
        .map_err(|e| ActionError::new("Failed to unpack pack", Some(format!("{:#?}", e))))?;

    Ok(timer.elapsed().unwrap_or_default())
}
