use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
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
    ConversionError,
    PathAlreadyExists,
    PathNotFound,
    PathError,
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// Pack is used to pack files and their paths in a single
/// file.
pub struct Pack(PathBuf);

impl Pack {
    pub fn new<P>(path: P) -> Pack
    where
        P: AsRef<Path>,
    {
        Pack(path.as_ref().to_owned())
    }

    /// Creates/Writes the pack file using the contents of the given
    /// directory.
    pub fn create<P>(&self, dir_path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
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

    /// Unpacks the pack from the associated pack file (fails
    /// if the pack file doesn't exist).
    pub fn unpack<P>(&self, output_path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
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

            let path = output_path
                .as_ref()
                .join(match std::str::from_utf8(&path_bytes) {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorKind::ConversionError),
                });

            let parent_dir_path = match path.parent() {
                Some(v) => v.to_path_buf(),
                None => return Err(ErrorKind::PathError),
            };
            if let Err(e) = fs::create_dir_all(parent_dir_path) {
                return Err(ErrorKind::IO(e));
            };
            let mut file = match OpenOptions::new().write(true).create(true).open(&path) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };

            let iterations =
                header
                    .file_len_u64()
                    .div_euclid(match DEFAULT_BUFFER_LEN.try_into() {
                        Ok(v) => v,
                        Err(_) => return Err(ErrorKind::ConversionError),
                    });
            let rem = header
                .file_len_u64()
                .rem_euclid(match DEFAULT_BUFFER_LEN.try_into() {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorKind::ConversionError),
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
                    Err(_) => return Err(ErrorKind::ConversionError),
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
    pub fn new<P1, P2>(file_path: P1, dir_path: P2) -> Result<FileHeader>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let path_str = match file_path.as_ref().strip_prefix(dir_path) {
            Ok(v) => match v.to_str() {
                Some(v) => v,
                None => return Err(ErrorKind::PathError),
            },
            Err(_) => return Err(ErrorKind::PathError),
        };

        // The length of the path (as bytes).
        let path_len = match u32::try_from(path_str.as_bytes().len()) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::ConversionError),
        };
        // The length of the file.
        let file_len = match fs::metadata(file_path.as_ref()) {
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
            Err(_) => return Err(ErrorKind::ConversionError),
        };
        let file_len_bytes: [u8; FILE_LEN_LEN] = match (&bytes[PATH_LEN_LEN..]).try_into() {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::ConversionError),
        };

        Ok(FileHeader {
            path: None,
            path_len: match u32::from_be_bytes(path_len_bytes).try_into() {
                Ok(v) => v,
                Err(_) => return Err(ErrorKind::ConversionError),
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
            Err(_) => return Err(ErrorKind::ConversionError),
        }
    }

    pub fn file_len_u64(&self) -> u64 {
        self.file_len
    }
}
