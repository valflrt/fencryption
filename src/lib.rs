//! This is the lib for the fencryption binary crate.
//!
//! Some Interesting structs:
//! - [Crypto][crate::crypto::Crypto]: Cryptography utility
//! struct
//! - [WalkDir][crate::walk_dir::WalkDir]: A struct for walking
//! through a directory
//! - [Pack][crate::pack::Pack]: A struct to create/unpack
//! packs
//! - [FileHeader][crate::file_header::FileHeader]: A struct
//! to create/parse file headers
//! - [TmpDir][crate::tmp_dir::TmpDir]: A struct to manipulate
//! temporary directories

pub mod constants;
pub mod crypto;
pub mod file_header;
pub mod pack;
pub mod tmp_dir;
pub mod walk_dir;

#[cfg(test)]
mod lib_tests;
