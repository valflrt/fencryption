//! This is the lib for the fencryption binary crate.
//!
//! Some Interesting structs:
//! - [crate::crypto::Crypto]: Cryptography utility
//! struct
//! - [crate::walk_dir::WalkDir]: A struct for walking
//! through a directory
//! - [crate::pack::Pack]: A struct to create/unpack packs
//! - [crate::file_header::FileHeader]: A struct to create/parse
//! file headers
//! - [crate::tmp_dir::TmpDir]: A struct to manipulate temporary
//! directories
//!
//! Modules:
//! - [crate::crypto]
//! - [crate::walk_dir]
//! - [crate::pack]
//! - [crate::file_header]
//! - [crate::tmp_dir]
//! - [crate::constants]

pub mod constants;
pub mod crypto;
pub mod file_header;
pub mod pack;
pub mod tmp_dir;
pub mod walk_dir;

mod test;
