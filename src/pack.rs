use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::{
    constants::DEFAULT_BUFFER_LEN,
    walk_dir::{self, WalkDir},
};

const PATH_LEN_LEN: usize = 32 / 8; // 4 bytes
const FILE_LEN_LEN: usize = 64 / 8; // 8 bytes
const HEADER_LEN: usize = PATH_LEN_LEN + FILE_LEN_LEN; // 12 bytes

// TODO Edit ErrorKind arms (they are ugly)
#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    WalkDir(walk_dir::ErrorKind),
    TryFromIntError,
    TryFromSliceError,
    Utf8Error,
    PathAlreadyExists,
    PathNotFound,
    PathError,
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// Pack is used to pack files and their paths in a single
/// file
pub struct Pack(PathBuf);

impl Pack {
    /// Creates a new pack
    pub fn new(path: &PathBuf) -> Pack {
        Pack(path.to_path_buf())
    }

    pub fn create(&self, dir_path: &PathBuf) -> Result<()> {
        let mut pack_file = match OpenOptions::new().write(true).create(true).open(&self.0) {
            Ok(v) => v,
            Err(e) => return Err(ErrorKind::IO(e)),
        };

        let walk_dir = match WalkDir::new(&dir_path) {
            Ok(v) => v,
            Err(e) => return Err(ErrorKind::WalkDir(e)),
        };

        for entry in walk_dir {
            let entry = match entry {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::WalkDir(e)),
            };

            if match entry.metadata() {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            }
            .is_file()
            {
                let mut file = match OpenOptions::new().read(true).open(entry.path()) {
                    Ok(v) => v,
                    Err(e) => return Err(ErrorKind::IO(e)),
                };

                // Creates file header.
                let header = match FileHeader::new(&entry.path(), &dir_path) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                // Writes file header to the pack.
                if let Err(e) = pack_file.write_all(&header.to_vec()?) {
                    return Err(ErrorKind::IO(e));
                };

                let mut buffer = [0u8; DEFAULT_BUFFER_LEN];
                loop {
                    let read_len = match file.read(&mut buffer) {
                        Ok(v) => v,
                        Err(e) => return Err(ErrorKind::IO(e)),
                    };

                    if let Err(e) = pack_file.write(&buffer[..read_len]) {
                        return Err(ErrorKind::IO(e));
                    };

                    if read_len != DEFAULT_BUFFER_LEN {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn unpack(&self) -> Result<()> {
        let mut pack_file = match OpenOptions::new().read(true).open(&self.0) {
            Ok(v) => v,
            Err(e) => return Err(ErrorKind::IO(e)),
        };

        loop {
            let mut header_bytes = [0u8; HEADER_LEN];
            let read_count = match pack_file.read(&mut header_bytes) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };
            if read_count != header_bytes.len() {
                break Ok(());
            };

            let header = match FileHeader::from_bytes(&header_bytes) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            let mut path_bytes = vec![0u8; header.path_len_usize()?];
            if let Err(e) = pack_file.read_exact(&mut path_bytes) {
                return Err(ErrorKind::IO(e));
            };

            // Creates file path
            let path = PathBuf::from(match self.0.file_stem() {
                Some(v) => v,
                None => return Err(ErrorKind::PathError),
            })
            .join(PathBuf::from(match std::str::from_utf8(&path_bytes) {
                Ok(v) => v,
                Err(_) => return Err(ErrorKind::Utf8Error),
            }));

            let parent_dir_path = match PathBuf::from(&path).parent() {
                Some(v) => v.to_path_buf(),
                None => return Err(ErrorKind::PathError),
            };
            if let Err(e) = fs::create_dir_all(parent_dir_path) {
                return Err(ErrorKind::IO(e));
            };
            let mut file = match OpenOptions::new().write(true).create(true).open(path) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };

            let iterations =
                header
                    .file_len_u64()
                    .div_euclid(match DEFAULT_BUFFER_LEN.try_into() {
                        Ok(v) => v,
                        Err(_) => return Err(ErrorKind::TryFromIntError),
                    });
            let rem = header
                .file_len_u64()
                .rem_euclid(match DEFAULT_BUFFER_LEN.try_into() {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorKind::TryFromIntError),
                });

            let mut buffer = [0u8; DEFAULT_BUFFER_LEN];
            for _ in 0..iterations {
                match pack_file.read_exact(&mut buffer) {
                    Ok(v) => v,
                    Err(e) => return Err(ErrorKind::IO(e)),
                };
                if let Err(e) = file.write_all(&buffer) {
                    return Err(ErrorKind::IO(e));
                };
            }

            let mut buffer = vec![
                0u8;
                match rem.try_into() {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorKind::TryFromIntError),
                }
            ];
            match pack_file.read_exact(&mut buffer) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };
            if let Err(e) = file.write_all(&buffer) {
                return Err(ErrorKind::IO(e));
            };
        }
    }

    pub fn path(&self) -> &PathBuf {
        return &self.0;
    }
}

/// Manage pack file headers.
///
/// A pack file header is made up of 12 bytes:
/// - 4 bytes representing the length of the associated path
/// - 8 bytes representing the length of the associated file
///
/// The maximum file length that can be stored in the
/// header is about 18.4 Terabytes.
pub struct FileHeader {
    path: Option<PathBuf>,
    path_len: u32,
    file_len: u64,
}

impl FileHeader {
    /// Creates a pack file header.
    pub fn new(file_path: &PathBuf, dir_path: &PathBuf) -> Result<FileHeader> {
        let path_str = match file_path.strip_prefix(dir_path) {
            Ok(v) => match v.to_str() {
                Some(v) => v,
                None => return Err(ErrorKind::PathError),
            },
            Err(_) => return Err(ErrorKind::PathError),
        };

        // The length of the path (as bytes).
        let path_len = match u32::try_from(path_str.as_bytes().len()) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::TryFromIntError),
        };
        // The length of the file.
        let file_len = match fs::metadata(file_path) {
            Ok(v) => v.len(),
            Err(e) => return Err(ErrorKind::IO(e)),
        };

        Ok(FileHeader {
            path: Some(PathBuf::from(path_str)),
            path_len,
            file_len,
        })
    }

    /// Extracts a file header from an array of 12 bytes.
    pub fn from_bytes(bytes: &[u8; 12]) -> Result<FileHeader> {
        let path_len_bytes: [u8; PATH_LEN_LEN] = match (&bytes[..PATH_LEN_LEN]).try_into() {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::TryFromSliceError),
        };
        let file_len_bytes: [u8; FILE_LEN_LEN] = match (&bytes[PATH_LEN_LEN..]).try_into() {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::TryFromSliceError),
        };

        Ok(FileHeader {
            path: None,
            path_len: match u32::from_be_bytes(path_len_bytes).try_into() {
                Ok(v) => v,
                Err(_) => return Err(ErrorKind::TryFromIntError),
            },
            file_len: u64::from_be_bytes(file_len_bytes),
        })
    }

    /// Returns the current header as a vector of bytes
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok([
            self.path_len.to_be_bytes().as_slice(),
            self.file_len.to_be_bytes().as_slice(),
            match &self.path {
                Some(v) => match v.to_str() {
                    Some(v) => v.as_bytes(),
                    None => return Err(ErrorKind::PathError),
                },
                None => return Err(ErrorKind::PathError),
            },
        ]
        .concat())
    }

    pub fn path_len_usize(&self) -> Result<usize> {
        match self.path_len.try_into() {
            Ok(v) => Ok(v),
            Err(_) => return Err(ErrorKind::TryFromIntError),
        }
    }

    pub fn file_len_u64(&self) -> u64 {
        self.file_len
    }
}
