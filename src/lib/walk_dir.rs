use std::{
    fs::{self, DirEntry, ReadDir},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum ErrorKind {
    IOError(io::Error),
}

pub type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// A struct for walking through a directory.
pub struct WalkDir(PathBuf);

impl WalkDir {
    pub fn new<P: AsRef<Path>>(path: P) -> WalkDir {
        WalkDir(path.as_ref().to_path_buf())
    }

    pub fn iter(&self) -> Result<WalkDirIterator> {
        let mut levels = Vec::new();
        levels.push(fs::read_dir(&self.0).map_err(|e| ErrorKind::IOError(e))?);
        Ok(WalkDirIterator { levels })
    }
}

/// The Iterator for WalkDir
pub struct WalkDirIterator {
    levels: Vec<ReadDir>,
}

impl Iterator for WalkDirIterator {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.levels.last_mut() {
            Some(current_dir) => match current_dir.next() {
                Some(Ok(entry)) => match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            self.levels.push(match fs::read_dir(&entry.path()) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(ErrorKind::IOError(e))),
                            });
                        }
                        return Some(Ok(entry));
                    }
                    Err(e) => return Some(Err(ErrorKind::IOError(e))),
                },
                Some(Err(e)) => return Some(Err(ErrorKind::IOError(e))),
                None => {
                    self.levels.pop();
                    return self.next();
                }
            },
            None => None,
        }
    }
}
