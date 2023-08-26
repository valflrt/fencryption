use std::{fmt::Debug, io, path::StripPrefixError, string::FromUtf8Error};

use fencryption_lib::metadata;

/// Command execution Result.
pub type Result<T, E = ErrorKind> = std::result::Result<T, E>;

#[derive(Debug, Default)]
pub enum ErrorKind {
    AtLeastOnePath,
    OnePathWhenOutputPathSet,

    FileNotFound,
    UnknownFileType,

    ReadKey(io::Error),
    ReadConfirmKey(io::Error),
    KeysNotMatching,
    NoKey,

    OutputAlreadyExists,

    CreateOutputDir(io::Error),
    CreateSubDir(io::Error),

    Overwrite(io::Error),
    DeleteOriginal(io::Error),

    ReadDir(io::Error),
    ReadDirEntry(io::Error),

    GetRelativePath(StripPrefixError),

    ReadSource(io::Error),
    OpenOrCreateDestination(io::Error),

    EncodeMetadata(metadata::ErrorKind),
    DecodeMetadata(metadata::ErrorKind),

    EncryptMetadata(io::Error),
    DecryptMetadata(io::Error),

    GetEncryptedMetadataLength(io::Error),
    GetEncryptedMetadata(io::Error),

    WriteMetadata(io::Error),

    EncryptText(io::Error),
    DecryptText(io::Error),

    EncryptFile(io::Error),
    DecryptFile(io::Error),
    EncryptFileIo(io::Error),
    DecryptFileIo(io::Error),

    DecodeBase64(base64::DecodeError),
    ConvertUtf8(FromUtf8Error),

    #[default]
    Unknown,
}

impl ErrorKind {
    pub fn to_string(&self, d: bool) -> String {
        match self {
            ErrorKind::AtLeastOnePath => "Please provide at least one path".to_string(),
            ErrorKind::OnePathWhenOutputPathSet => {
                "Please provide exactly one path when an output path is set".to_string()
            }
            ErrorKind::FileNotFound => "I can't work with files that don't exist".to_string(),
            ErrorKind::UnknownFileType => "Unknown file type".to_string(),
            ErrorKind::ReadKey(e) => format_message("Failed to read key", e, d),
            ErrorKind::ReadConfirmKey(e) => format_message("Failed to read confirm key", e, d),
            ErrorKind::KeysNotMatching => "The two keys don't match".to_string(),
            ErrorKind::NoKey => "You must set a key".to_string(),
            ErrorKind::OutputAlreadyExists => {
                "The output file/directory already exists (use --overwrite / -O to force overwrite)"
                    .to_string()
            }
            ErrorKind::CreateOutputDir(e) => {
                format_message("Failed to create output directory", e, d)
            }
            ErrorKind::CreateSubDir(e) => format_message("Failed to create sub-directory", e, d),
            ErrorKind::Overwrite(e) => format_message(
                "Failed to overwrite file/directory, please do it yourself",
                e,
                d,
            ),
            ErrorKind::DeleteOriginal(e) => format_message(
                "Failed to delete original file/directory, please do it yourself",
                e,
                d,
            ),
            ErrorKind::ReadDir(e) => format_message("Failed to read directory", e, d),
            ErrorKind::ReadDirEntry(e) => format_message("Failed to read directory entry", e, d),
            ErrorKind::GetRelativePath(e) => {
                format_message("Failed to get relative entry path", e, d)
            }
            ErrorKind::ReadSource(e) => format_message("Failed to read source file", e, d),
            ErrorKind::OpenOrCreateDestination(e) => {
                format_message("Failed to open/create destination file", e, d)
            }
            ErrorKind::EncodeMetadata(e) => format_message("Failed to encode file metadata", e, d),
            ErrorKind::DecodeMetadata(e) => format_message("Failed to decode metadata", e, d),
            ErrorKind::EncryptMetadata(e) => format_message("Failed to encrypt metadata", e, d),
            ErrorKind::DecryptMetadata(e) => format_message("Failed to decrypt metadata", e, d),
            ErrorKind::GetEncryptedMetadataLength(e) => {
                format_message("Failed to get encrypted metadata length", e, d)
            }
            ErrorKind::GetEncryptedMetadata(e) => {
                format_message("Failed to get encrypted metadata", e, d)
            }
            ErrorKind::WriteMetadata(e) => format_message("Failed to write metadata", e, d),
            ErrorKind::EncryptFile(e) => {
                format_message("Failed to encrypt file (key must be wrong)", e, d)
            }
            ErrorKind::DecryptFile(e) => {
                format_message("Failed to decrypt file (key must be wrong)", e, d)
            }
            ErrorKind::EncryptFileIo(e) => format_message("Failed to encrypt file", e, d),
            ErrorKind::DecryptFileIo(e) => format_message("Failed to decrypt file", e, d),
            ErrorKind::EncryptText(e) => format_message("Failed to encrypt text", e, d),
            ErrorKind::DecryptText(e) => format_message("Failed to decrypt text", e, d),
            ErrorKind::DecodeBase64(e) => format_message("Failed to decode base64", e, d),
            ErrorKind::ConvertUtf8(e) => {
                format_message("Failed to convert decrypted bytes to utf8", e, d)
            }
            ErrorKind::Unknown => "Unknown error".to_string(),
        }
    }
}

fn format_message<E>(message: &'static str, error: E, debug_mode: bool) -> String
where
    E: Debug,
{
    format!(
        "{}{}",
        message,
        if debug_mode {
            format!("\n{:#?}", error)
        } else {
            "".to_string()
        }
    )
}
