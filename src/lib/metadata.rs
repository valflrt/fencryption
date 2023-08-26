//! Encode and decode metadata.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// Enum of the different possible metadata errors.
#[derive(Debug)]
pub enum ErrorKind {
    SerializeError(rmp_serde::encode::Error),
    DeserializeError(rmp_serde::decode::Error),
}

type Result<T, E = ErrorKind> = std::result::Result<T, E>;

/// Encode [`serde::Serialize`] implementor into bytes.
pub fn encode<D>(data: D) -> Result<Vec<u8>>
where
    D: Serialize,
{
    rmp_serde::to_vec(&data).map_err(ErrorKind::SerializeError)
}

/// Decode bytes into specified [`serde::Deserialize`]
/// implementor.
pub fn decode<O>(bytes: &[u8]) -> Result<O>
where
    O: for<'a> Deserialize<'a>,
{
    rmp_serde::from_slice::<O>(bytes).map_err(ErrorKind::DeserializeError)
}
