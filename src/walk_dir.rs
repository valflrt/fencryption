use std::{
    fs::{self, DirEntry, ReadDir},
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub enum ErrorKind {
    IOError(io::Error),
}

pub type Result<T, E = ErrorKind> = std::result::Result<T, E>;

pub struct WalkDir {
    levels: Vec<ReadDir>,
}

impl WalkDir {
    pub fn new(path: &PathBuf) -> Result<WalkDir> {
        let mut levels = Vec::new();
        levels.push(match fs::read_dir(&path) {
            Ok(v) => v,
            Err(e) => return Err(ErrorKind::IOError(e)),
        });
        Ok(WalkDir { levels })
    }
}

impl Iterator for WalkDir {
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
