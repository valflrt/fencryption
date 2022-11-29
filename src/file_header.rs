use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    ConversionError,
    PathError,
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

pub const PATH_LEN_LEN: usize = 32 / 8; // 4 bytes
pub const FILE_LEN_LEN: usize = 64 / 8; // 8 bytes
pub const HEADER_LEN: usize = PATH_LEN_LEN + FILE_LEN_LEN; // 12 bytes

/// Manipulate (create/parse) file headers.
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
    ///
    /// The dir_path argument is used to determine the relative
    /// file path.
    pub fn new<P1, P2>(file_path: P1, dir_path: P2) -> Result<Self>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let path_str = file_path
            .as_ref()
            .strip_prefix(dir_path)
            .map_err(|_| ErrorKind::PathError)?
            .to_str()
            .ok_or(ErrorKind::PathError)?;

        // The length of the path (as bytes).
        let path_len = path_str.as_bytes().len() as u32;
        // The length of the file.
        let file_len = fs::metadata(file_path.as_ref())
            .map_err(|e| ErrorKind::IO(e))?
            .len();

        Ok(FileHeader {
            path: Some(PathBuf::from(path_str)),
            path_len,
            file_len,
        })
    }

    /// Extracts a file header from an array of 12 bytes.
    pub fn from_bytes(bytes: &[u8; 12]) -> Result<Self> {
        let path_len_bytes: [u8; PATH_LEN_LEN] = bytes[..PATH_LEN_LEN]
            .try_into()
            .map_err(|_| ErrorKind::ConversionError)?;
        let file_len_bytes: [u8; FILE_LEN_LEN] = bytes[PATH_LEN_LEN..]
            .try_into()
            .map_err(|_| ErrorKind::ConversionError)?;

        Ok(FileHeader {
            path: None,
            path_len: u32::from_be_bytes(path_len_bytes)
                .try_into()
                .map_err(|_| ErrorKind::ConversionError)?,
            file_len: u64::from_be_bytes(file_len_bytes),
        })
    }

    /// Returns the current header as a vector of bytes
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok([
            self.path_len.to_be_bytes().as_slice(),
            self.file_len.to_be_bytes().as_slice(),
            self.path
                .as_ref()
                .ok_or(ErrorKind::PathError)?
                .to_str()
                .ok_or(ErrorKind::PathError)?
                .as_bytes(),
        ]
        .concat())
    }

    pub fn path_len(&self) -> u32 {
        self.path_len
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }
}
