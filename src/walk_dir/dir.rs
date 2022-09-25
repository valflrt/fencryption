use std::{
    fs::{self, DirEntry, ReadDir},
    path::PathBuf,
};

use super::{Result, WalkDirErrorKind};

#[derive(Debug)]
pub struct Dir {
    read_dir: ReadDir,
    consumed: bool,
}

impl Dir {
    pub fn new(path: PathBuf) -> Result<Dir> {
        let read_dir = match fs::read_dir(&path) {
            Ok(v) => v,
            Err(e) => return Err(WalkDirErrorKind::IOError(e)),
        };
        Ok(Dir {
            read_dir,
            consumed: false,
        })
    }
}

impl Iterator for Dir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_dir.next() {
            Some(Ok(v)) => Some(Ok(v)),
            Some(Err(e)) => Some(Err(WalkDirErrorKind::IOError(e))),
            None => {
                self.consumed = true;
                None
            }
        }
    }
}
