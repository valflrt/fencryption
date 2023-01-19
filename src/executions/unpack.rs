use std::{
    path::PathBuf,
    time::{self, Duration},
};

use fencryption_lib::{crypto::Crypto, pack::Pack, tmp::TmpDir};

use crate::{
    actions,
    executions::{ActionError, ActionResult},
};

pub fn unpack(
    input_path: PathBuf,
    output_dir_path: PathBuf,
    key: String,
) -> ActionResult<Duration> {
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| ActionError::new("Failed to create cipher").error(e))?;

    let tmp_dir = TmpDir::new()
        .map_err(|e| ActionError::new("Failed to create temporary directory").error(e))?;

    let tmp_pack_path = tmp_dir.unique_path();

    let (mut source, mut dest) = actions::create_files(&input_path, &tmp_pack_path)?;

    crypto
        .decrypt_stream(&mut source, &mut dest)
        .map_err(|e| ActionError::new("Failed to decrypt pack").error(e))?;

    Pack::new(&tmp_pack_path)
        .unpack(&output_dir_path)
        .map_err(|e| ActionError::new("Failed to unpack pack").error(e))?;

    Ok(timer.elapsed().unwrap_or_default())
}
