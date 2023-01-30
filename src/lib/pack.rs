//! Create/unpack packs.
//!
//! A pack is a file with all the contents of a directory
//! inside of it.
//!
//! Uses [`metadata`] in order to ease the process of
//! storing/separating the files inside of it.

use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    io::{stream, DEFAULT_BUF_LEN},
    metadata,
    walk_dir::walk_dir,
};

#[derive(Serialize, Deserialize, Debug)]
struct PackEntryMetadata(PathBuf, u64);

impl PackEntryMetadata {
    pub fn new<P>(path: P, file_len: u64) -> Self
    where
        P: AsRef<Path>,
    {
        PackEntryMetadata(path.as_ref().to_owned(), file_len)
    }

    pub fn path(&self) -> PathBuf {
        self.0.to_owned()
    }

    pub fn file_len(&self) -> u64 {
        self.1
    }
}

/// Enum of the different possible pack errors.
#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    MetadataError(metadata::ErrorKind),
    ConversionError,
    PathAlreadyExists,
    PathNotFound,
    PathError,
}
type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// Create the pack file with the contents of the specified
/// directory.
pub fn create<P1, P2>(input_dir_path: P1, output_dir_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut dest = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_dir_path.as_ref())
        .map_err(|e| ErrorKind::Io(e))?;

    let walk_dir = walk_dir(&input_dir_path).map_err(|e| ErrorKind::Io(e))?;

    for entry in walk_dir {
        let entry = entry.map_err(|e| ErrorKind::Io(e))?;

        if entry.path().is_file() {
            let mut source = OpenOptions::new()
                .read(true)
                .open(entry.path())
                .map_err(|e| ErrorKind::Io(e))?;

            // Creates file header.
            let metadata = metadata::encode(PackEntryMetadata::new(
                input_dir_path
                    .as_ref()
                    .strip_prefix(entry.path())
                    .map_err(|_| ErrorKind::PathError)?,
                entry.metadata().map_err(|e| ErrorKind::Io(e))?.len(),
            ))
            .map_err(|e| ErrorKind::MetadataError(e))?;

            // Writes file header to the pack.
            dest.write_all(
                &[
                    (metadata.len() as u16).to_be_bytes().as_ref(),
                    metadata.as_ref(),
                ]
                .concat(),
            )
            .map_err(|e| ErrorKind::Io(e))?;

            stream(&mut source, &mut dest).map_err(|e| ErrorKind::Io(e))?;
        }
    }

    Ok(())
}

/// Unpack the pack from the associated pack file (fails
/// if the pack file doesn't exist).
pub fn unpack<P1, P2>(input_dir_path: P1, output_dir_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let mut source = OpenOptions::new()
        .read(true)
        .open(input_dir_path.as_ref())
        .map_err(|e| ErrorKind::Io(e))?;

    loop {
        let mut len_bytes = [0u8; 2];
        source
            .read_exact(&mut len_bytes)
            .map_err(|e| ErrorKind::Io(e))?;
        let len = u16::from_be_bytes(len_bytes) as usize;
        let mut metadata_bytes = vec![0u8; len];
        source
            .read_exact(&mut metadata_bytes)
            .map_err(|e| ErrorKind::Io(e))?;
        let metadata = metadata::decode::<PackEntryMetadata>(&metadata_bytes)
            .map_err(|e| ErrorKind::MetadataError(e))?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_dir_path.as_ref().join(metadata.path()))
            .map_err(|e| ErrorKind::Io(e))?;

        let file_len = metadata.file_len();

        // Gets the number of chunks to read before reaching
        // the last bytes.
        let chunks = file_len.div_euclid(DEFAULT_BUF_LEN as u64);
        // Gets the number of the remaining bytes (after
        // reading all chunks).
        let rem_len = file_len.rem_euclid(DEFAULT_BUF_LEN as u64) as usize;

        // Reads all chunks and writes them to the output
        // file.
        let mut buffer = [0u8; DEFAULT_BUF_LEN];
        for _ in 0..chunks {
            source
                .read_exact(&mut buffer)
                .map_err(|e| ErrorKind::Io(e))?;
            file.write_all(&buffer).map_err(|e| ErrorKind::Io(e))?;
        }

        // Reads the remaining bytes and writes them to
        // the output file.
        let mut last = vec![0u8; rem_len];
        source.read_exact(&mut last).map_err(|e| ErrorKind::Io(e))?;
        file.write_all(&last).map_err(|e| ErrorKind::Io(e))?;
    }
}
