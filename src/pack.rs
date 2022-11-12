use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    constants::DEFAULT_BUF_LEN,
    file_header::{self, FileHeader, HEADER_LEN},
    walk_dir::{self, WalkDir},
};

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    WalkDir(walk_dir::ErrorKind),
    FileHeader(file_header::ErrorKind),
    ConversionError,
    PathAlreadyExists,
    PathNotFound,
    PathError,
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// A struct to manipulate (create/unpack) packs.
///
/// A pack is a file with all the contents of a directory
/// inside of it.
///
/// Pack uses [FileHeader][crate::file_header::FileHeader] in order to
/// easily store/separate files inside of it.
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

        let walk_dir = WalkDir::new(&dir_path)
            .iter()
            .map_err(|e| ErrorKind::WalkDir(e))?;

        for entry in walk_dir {
            let entry = entry.map_err(|e| ErrorKind::WalkDir(e))?;

            if entry.path().is_file() {
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(entry.path())
                    .map_err(|e| ErrorKind::IO(e))?;

                // Creates file header.
                let header = FileHeader::new(&entry.path(), &dir_path)
                    .map_err(|e| ErrorKind::FileHeader(e))?;

                // Writes file header to the pack.
                pack_file
                    .write_all(&header.to_vec().map_err(|e| ErrorKind::FileHeader(e))?)
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

            let header =
                FileHeader::try_from(&header_bytes).map_err(|e| ErrorKind::FileHeader(e))?;

            let mut path_bytes = vec![
                0u8;
                header
                    .path_len_usize()
                    .map_err(|e| ErrorKind::FileHeader(e))?
            ];
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
