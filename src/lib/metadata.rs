use std::{
    collections::TryReserveError,
    fs::File,
    io::{self, Read},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ErrorKind {
    SerializeError(rmp_serde::encode::Error),
    DeserializeError(rmp_serde::decode::Error),
    IoError(io::Error),
    DecodeInt,
    TryReserveError(TryReserveError),
}

pub type Result<T, E = ErrorKind> = std::result::Result<T, E>;

pub fn encode<D>(data: D) -> Result<Vec<u8>>
where
    D: Serialize,
{
    let vec = rmp_serde::to_vec(&data).map_err(|e| ErrorKind::SerializeError(e))?;
    Ok([(vec.len() as u32).to_be_bytes().as_ref(), &vec].concat())
}

pub fn get_metadata<O>(file: &mut File) -> Result<O>
where
    O: for<'a> Deserialize<'a>,
{
    let mut len_bytes = [0u8; 4];
    file.read_exact(&mut len_bytes)
        .map_err(|e| ErrorKind::IoError(e))?;
    let len = u32::from_be_bytes(len_bytes);

    let mut data_bytes = vec![0u8; len as usize];
    file.read_exact(&mut data_bytes)
        .map_err(|e| ErrorKind::IoError(e))?;

    Ok(rmp_serde::from_slice::<O>(&data_bytes.to_vec())
        .map_err(|e| ErrorKind::DeserializeError(e))?)
}
