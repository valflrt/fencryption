use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use crate::constants::DEFAULT_BUFFER_LEN;

const PATH_LEN_LEN: usize = 32 / 8; // 4 bytes
const FILE_LEN_LEN: usize = 64 / 8; // 8 bytes
const HEADER_LEN: usize = PATH_LEN_LEN + FILE_LEN_LEN; // 12 bytes

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    TryFromIntError,
    TryFromSliceError,
    Utf8Error,
    PathAlreadyExists,
    PathNotFound,
    PathError,
}

type Result<T> = std::result::Result<T, ErrorKind>;

// TODO make stored file paths relative to the pack path

/// Pack is used to pack files and their paths in a single
/// file
pub struct Pack {
    path: PathBuf,
}

impl Pack {
    /// Creates a new pack
    pub fn new(path: PathBuf) -> Pack {
        Pack { path }
    }

    /// Creates a PackWriter, consumes self.
    pub fn writer(self) -> Result<PackWriter> {
        if self.path.exists() {
            return Err(ErrorKind::PathAlreadyExists);
        }
        Ok(PackWriter::new(self)?)
    }

    /// Creates a PackReader, consumes self.
    pub fn reader(self) -> Result<PackReader> {
        if !self.path.exists() {
            return Err(ErrorKind::PathNotFound);
        }
        Ok(PackReader::new(self)?)
    }

    pub fn path(&self) -> &PathBuf {
        return &self.path;
    }
}

pub struct PackWriter {
    file: File,
}

impl PackWriter {
    /// Creates a new PackWriter
    pub fn new(pack: Pack) -> Result<PackWriter> {
        Ok(PackWriter {
            file: match OpenOptions::new().write(true).create(true).open(pack.path) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            },
        })
    }

    /// Adds a file to the pack
    pub fn add(&mut self, file_path: &str) -> Result<()> {
        let mut file = match OpenOptions::new().read(true).open(file_path) {
            Ok(v) => v,
            Err(e) => return Err(ErrorKind::IO(e)),
        };

        // Creates file header.
        let header = match FileHeader::new(file_path) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        // Writes file header to the pack.
        if let Err(e) = self
            .file
            .write_all(&[&header.to_vec()?, file_path.as_bytes()].concat())
        {
            return Err(ErrorKind::IO(e));
        };

        let mut buffer = [0u8; DEFAULT_BUFFER_LEN];
        loop {
            let read_len = match file.read(&mut buffer) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };

            if let Err(e) = self.file.write(&buffer[..read_len]) {
                return Err(ErrorKind::IO(e));
            };

            if read_len != DEFAULT_BUFFER_LEN {
                break;
            }
        }

        Ok(())
    }
}

pub struct PackReader {
    file: File,
}

impl PackReader {
    /// Creates a new PackReader
    pub fn new(pack: Pack) -> Result<PackReader> {
        Ok(PackReader {
            file: match OpenOptions::new().read(true).open(pack.path) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            },
        })
    }

    /// Write all the files to their associated path
    pub fn unpack_all(&mut self) -> Result<()> {
        loop {
            let mut header_bytes = [0u8; HEADER_LEN];
            let read_count = match self.file.read(&mut header_bytes) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };
            if read_count != header_bytes.len() {
                break Ok(());
            }

            println!("header bytes: {:x?}", header_bytes);

            println!("make header");
            let header = match FileHeader::from_bytes(&header_bytes) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            println!("read path bytes");
            let mut path_bytes = vec![0u8; header.path_len_usize()?];
            if let Err(e) = self.file.read_exact(&mut path_bytes) {
                return Err(ErrorKind::IO(e));
            };

            println!("make path");
            let path = match std::str::from_utf8(&path_bytes) {
                Ok(v) => v,
                Err(_) => return Err(ErrorKind::Utf8Error),
            };

            let parent_dir_path = match PathBuf::from(path).parent() {
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

            println!("iter and rem");
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

            println!("buffer1");
            let mut buffer = [0u8; DEFAULT_BUFFER_LEN];
            for _ in 0..iterations {
                match self.file.read_exact(&mut buffer) {
                    Ok(v) => v,
                    Err(e) => return Err(ErrorKind::IO(e)),
                };
                if let Err(e) = file.write_all(&buffer) {
                    return Err(ErrorKind::IO(e));
                };
            }

            println!("buffer2");
            let mut buffer = vec![
                0u8;
                match rem.try_into() {
                    Ok(v) => v,
                    Err(_) => return Err(ErrorKind::TryFromIntError),
                }
            ];
            match self.file.read_exact(&mut buffer) {
                Ok(v) => v,
                Err(e) => return Err(ErrorKind::IO(e)),
            };
            if let Err(e) = file.write_all(&buffer) {
                return Err(ErrorKind::IO(e));
            };
        }
    }
}

/// Manage pack file headers.
///
/// A pack file header is made up of 12 bytes:
/// - 4 bytes representing the length of the associated path
/// - 8 bytes representing the length of the associated file.
///
/// The maximum file length that can be stored in the
/// header is about 18.4 Terabytes.
pub struct FileHeader {
    path_len: u32,
    file_len: u64,
}

impl FileHeader {
    /// Creates a pack file header.
    pub fn new(path: &str) -> Result<FileHeader> {
        // The length of the path (as bytes).
        let path_len = match u32::try_from(path.as_bytes().len()) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::TryFromIntError),
        };
        // The length of the file.
        let file_len = match fs::metadata(path) {
            Ok(v) => v.len(),
            Err(e) => return Err(ErrorKind::IO(e)),
        };

        println!("path: {:x?} ({})", path_len.to_be_bytes(), path.len());
        println!("file: {:x?} ({})", file_len.to_be_bytes(), file_len);

        Ok(FileHeader { path_len, file_len })
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
