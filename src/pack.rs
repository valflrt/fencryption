use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    constants::DEFAULT_BUF_LEN,
    walk_dir::{self, WalkDir},
};

const PATH_LEN_LEN: usize = 32 / 8; // 4 bytes
const FILE_LEN_LEN: usize = 64 / 8; // 8 bytes
const HEADER_LEN: usize = PATH_LEN_LEN + FILE_LEN_LEN; // 12 bytes

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
        let mut pack_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.0)
            .map_err(|e| ErrorKind::IO(e))?;

        let walk_dir = WalkDir::new(&dir_path).map_err(|e| ErrorKind::WalkDir(e))?;

        for entry in walk_dir {
            let entry = entry.map_err(|e| ErrorKind::WalkDir(e))?;

            if entry.path().is_file() {
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(entry.path())
                    .map_err(|e| ErrorKind::IO(e))?;

                // Creates file header.
                let header = FileHeader::new(&entry.path(), &dir_path)?;

                // Writes file header to the pack.
                pack_file
                    .write_all(&header.to_vec()?)
                    .map_err(|e| ErrorKind::IO(e))?;

                let mut buffer = [0u8; DEFAULT_BUF_LEN];
                loop {
                    let read_len = file.read(&mut buffer).map_err(|e| ErrorKind::IO(e))?;
                    pack_file
                        .write(&buffer[..read_len])
                        .map_err(|e| ErrorKind::IO(e))?;

                    if read_len != DEFAULT_BUF_LEN {
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
        let mut pack_file = OpenOptions::new()
            .read(true)
            .open(&self.0)
            .map_err(|e| ErrorKind::IO(e))?;

        loop {
            let mut header_bytes = [0u8; HEADER_LEN];

            let read_count = pack_file
                .read(&mut header_bytes)
                .map_err(|e| ErrorKind::IO(e))?;
            if read_count != header_bytes.len() {
                break Ok(());
            };

            let header = FileHeader::from_bytes(&header_bytes)?;

            let mut path_bytes = vec![0u8; header.path_len_usize()?];
            pack_file
                .read_exact(&mut path_bytes)
                .map_err(|e| ErrorKind::IO(e))?;

            let path = output_path
                .as_ref()
                .join(std::str::from_utf8(&path_bytes).map_err(|_| ErrorKind::ConversionError)?);

            // Creates all parent directories.
            let parent_dir_path = path.parent().ok_or(ErrorKind::PathError)?.to_path_buf();
            fs::create_dir_all(parent_dir_path).map_err(|e| ErrorKind::IO(e))?;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)
                .map_err(|e| ErrorKind::IO(e))?;

            // Converts the default buffer length into u64.
            let buf_len_u64 = DEFAULT_BUF_LEN
                .try_into()
                .map_err(|_| ErrorKind::ConversionError)?;

            // Gets the number of chunks to read before reaching
            // the last bytes.
            let chunks = header.file_len_u64().div_euclid(buf_len_u64);
            // Gets the number of the remaining bytes (after
            // reading all chunks).
            let rem_len = usize::try_from(header.file_len_u64().rem_euclid(buf_len_u64))
                .map_err(|_| ErrorKind::ConversionError)?;

            // Reads all chunks and writes them to the output
            // file.
            let mut buffer = [0u8; DEFAULT_BUF_LEN];
            for _ in 0..chunks {
                pack_file
                    .read_exact(&mut buffer)
                    .map_err(|e| ErrorKind::IO(e))?;
                file.write_all(&buffer).map_err(|e| ErrorKind::IO(e))?;
            }

            // Reads the remaining bytes and writes them to
            // the output file.
            let mut last = vec![0u8; rem_len];
            pack_file
                .read_exact(&mut last)
                .map_err(|e| ErrorKind::IO(e))?;
            file.write_all(&last).map_err(|e| ErrorKind::IO(e))?;
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
        let path_str = file_path
            .as_ref()
            .strip_prefix(dir_path)
            .map_err(|_| ErrorKind::PathError)?
            .to_str()
            .ok_or(ErrorKind::PathError)?;

        // The length of the path (as bytes).
        let path_len =
            u32::try_from(path_str.as_bytes().len()).map_err(|_| ErrorKind::ConversionError)?;
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
    pub fn from_bytes(bytes: &[u8; 12]) -> Result<FileHeader> {
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

    pub fn path_len_usize(&self) -> Result<usize> {
        Ok(usize::try_from(self.path_len).map_err(|_| ErrorKind::ConversionError)?)
    }

    pub fn file_len_u64(&self) -> u64 {
        self.file_len
    }
}
