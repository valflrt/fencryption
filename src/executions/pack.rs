use std::{
    path::PathBuf,
    time::{self, Duration},
};

use fencryption_lib::{crypto::Crypto, pack::Pack, tmp::TmpDir};

use crate::{
    actions,
    executions::{ActionError, ActionResult},
};

pub fn pack(
    input_path: PathBuf,
    key: String,
    overwrite: bool,
    delete_original: bool,
) -> ActionResult<Duration> {
    let timer = time::SystemTime::now();

    let crypto = Crypto::new(&key.as_bytes())
        .map_err(|e| ActionError::new("Failed to create cipher").error(e))?;

    let tmp_dir = TmpDir::new()
        .map_err(|e| ActionError::new("Failed to create temporary directory").error(e))?;

    let tmp_pack_path = tmp_dir.unique_path();
    let pack_path = actions::get_pack_path(&input_path);

    if pack_path.exists() {
        if !overwrite {
            return Err(ActionError::new("Pack already exists"));
        }
    }

    actions::overwrite_when_asked(&input_path, overwrite, actions::Command::Pack)?;

    Pack::new(&tmp_pack_path)
        .create(&input_path)
        .map_err(|e| ActionError::new("Failed to create pack").error(e))?;

    actions::delete_original_when_asked(&input_path, delete_original, actions::Command::Pack)?;

    let (mut source, mut dest) = actions::create_files(&tmp_pack_path, &pack_path)?;

    crypto
        .encrypt_stream(&mut source, &mut dest)
        .map_err(|e| ActionError::new("Failed to encrypt pack").error(e))?;

    Ok(timer.elapsed().unwrap_or_default())
}
