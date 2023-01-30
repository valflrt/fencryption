//! Encode and decode metadata.

use std::{collections::TryReserveError, fmt::Debug, io};

use serde::{Deserialize, Serialize};

/// Enum of the different possible metadata errors.
#[derive(Debug)]
pub enum ErrorKind {
    SerializeError(rmp_serde::encode::Error),
    DeserializeError(rmp_serde::decode::Error),
    IoError(io::Error),
    DecodeInt,
    TryReserveError(TryReserveError),
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// Encode [`serde::Serialize`] implementor into bytes.
pub fn encode<D>(data: D) -> Result<Vec<u8>>
where
    D: Serialize,
{
    Ok(rmp_serde::to_vec(&data).map_err(|e| ErrorKind::SerializeError(e))?)
}

/// Decode bytes into specified [`serde::Deserialize`] implementor.
pub fn decode<O>(bytes: &[u8]) -> Result<O>
where
    O: for<'a> Deserialize<'a>,
{
    Ok(rmp_serde::from_slice::<O>(&bytes.to_vec()).map_err(|e| ErrorKind::DeserializeError(e))?)
}
