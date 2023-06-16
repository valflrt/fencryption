//! Walk through a directory.

use std::{
    fs::{self, DirEntry, ReadDir},
    io::Result,
    path::Path,
};

/// Walk through the specified directory.
pub fn walk_dir<P>(path: P) -> Result<WalkDirIterator>
where
    P: AsRef<Path>,
{
    Ok(WalkDirIterator {
        levels: Vec::from([fs::read_dir(path.as_ref())?]),
    })
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
                            self.levels.push(match fs::read_dir(entry.path()) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            });
                        }
                        Some(Ok(entry))
                    }
                    Err(e) => Some(Err(e)),
                },
                Some(Err(e)) => Some(Err(e)),
                None => {
                    self.levels.pop();
                    self.next()
                }
            },
            None => None,
        }
    }
}
