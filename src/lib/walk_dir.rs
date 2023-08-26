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
                Some(entry_result) => Some(entry_result.and_then(|entry| {
                    entry.file_type().and_then(|file_type| {
                        if file_type.is_dir() {
                            self.levels.push(fs::read_dir(entry.path())?);
                        }
                        Ok(entry)
                    })
                })),
                None => {
                    self.levels.pop();
                    self.next()
                }
            },
            None => None,
        }
    }
}
