mod dir;
mod dir_array;

use dir::Dir;
use dir_array::DirArray;

use std::{fs::DirEntry, io, path::PathBuf};

#[derive(Debug)]
pub enum WalkDirErrorKind {
    IOError(io::Error),
}

pub type Result<T, E = WalkDirErrorKind> = std::result::Result<T, E>;

pub struct WalkDir {
    dir_chain: DirArray,
}

impl WalkDir {
    pub fn new(path: &PathBuf) -> Result<WalkDir> {
        Ok(WalkDir {
            dir_chain: DirArray::new(match Dir::new(path) {
                Ok(v) => v,
                Err(e) => return Err(e),
            }),
        })
    }
}

impl Iterator for WalkDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dir_chain.current() {
            Some(level) => match level.next() {
                Some(Ok(entry)) => match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            self.dir_chain.push(match Dir::new(&entry.path()) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            });
                        }
                        return Some(Ok(entry));
                    }
                    Err(e) => return Some(Err(WalkDirErrorKind::IOError(e))),
                },
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    self.dir_chain.pop();
                    return self.next();
                }
            },
            None => None,
        }
    }

    // fn next(&mut self) -> Option<Self::Item> {
    //     if let Some(current_dir) = self.current_dir.next() {
    //         match current_dir {
    //             Ok(entry) => match entry.file_type() {
    //                 Ok(file_type) => {
    //                     if file_type.is_file() {
    //                         return Some(Ok(entry));
    //                     } else if file_type.is_dir() {
    //                         self.current_path = entry.path();
    //                         let reader = match fs::read_dir(entry.path()) {
    //                             Ok(v) => v,
    //                             Err(e) => return Some(Err(Some(WalkDirErrorKind::IOError(e)))),
    //                         };
    //                         self.current_dir = reader;
    //                         loop {
    //                             match self.next() {
    //                                 Some(Ok(v)) => return Some(Ok(v)),
    //                                 Some(Err(e)) => return Some(Err(e)),
    //                                 None => (),
    //                             }
    //                         }
    //                     } else {
    //                         loop {
    //                             match self.next() {
    //                                 Some(Ok(v)) => return Some(Ok(v)),
    //                                 Some(Err(e)) => return Some(Err(e)),
    //                                 None => (),
    //                             }
    //                         }
    //                     }
    //                 }
    //                 Err(e) => Some(Err(Some(WalkDirErrorKind::IOError(e)))),
    //             },
    //             Err(e) => Some(Err(Some(WalkDirErrorKind::IOError(e)))),
    //         }
    //     } else {
    //         if let false = self.current_path.pop() {
    //             return None;
    //         };
    //         let reader = match fs::read_dir(&self.current_path) {
    //             Ok(v) => v,
    //             Err(e) => return Some(Err(Some(WalkDirErrorKind::IOError(e)))),
    //         };
    //         self.current_dir = reader;
    //         loop {
    //             match self.next() {
    //                 Some(Ok(v)) => return Some(Ok(v)),
    //                 Some(Err(e)) => return Some(Err(e)),
    //                 None => (),
    //             }
    //         }
    //     }
    // }
}
