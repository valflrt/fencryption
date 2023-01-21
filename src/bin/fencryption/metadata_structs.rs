use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileMetadata(PathBuf);

impl FileMetadata {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        FileMetadata(path.as_ref().to_owned())
    }

    pub fn path(&self) -> PathBuf {
        self.0.to_owned()
    }
}
