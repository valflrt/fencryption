mod decrypt;
mod encrypt;
mod pack;
mod unpack;

use std::path::PathBuf;

pub use decrypt::decrypt;
pub use encrypt::encrypt;
pub use pack::pack;
pub use unpack::unpack;

pub enum ErrorKind {
    CryptoInitError,
    EncryptError,
    DecryptError,
    CreateDirError(PathBuf),
    CreateFileError(PathBuf),
    CreateTmpDirError,
    CreateTmpFileError,
    OverwriteError(PathBuf),
    DeleteOriginalError(PathBuf),
}

pub struct ErrorBuilder {
    message: String,
    debug: Option<String>,
}

impl ErrorBuilder {
    pub fn new<M>(message: M) -> Self
    where
        M: AsRef<str>,
    {
        ErrorBuilder {
            message: message.as_ref().to_string(),
            debug: None,
        }
    }

    pub fn error(mut self, error: ErrorKind) -> Self {
        self.message = match error {
            ErrorKind::CryptoInitError => "Failed to initialize crypto".to_string(),
            ErrorKind::EncryptError => "Failed to encrypt".to_string(),
            ErrorKind::DecryptError => "Failed to decrypt".to_string(),
            ErrorKind::CreateDirError(p) => format!("Failed to create directory ({})", p.display()),
            ErrorKind::CreateFileError(p) => format!("Failed to create file ({})", p.display()),
            ErrorKind::CreateTmpDirError => "Failed to create temporary directory".to_string(),
            ErrorKind::CreateTmpFileError => "Failed to create temporary file".to_string(),
            ErrorKind::OverwriteError(p) => format!("Failed to overwrite ({})", p.display()),
            ErrorKind::DeleteOriginalError(p) => {
                format!("Failed to delete original file/directory ({})", p.display())
            }
        };
        return self;
    }

    pub fn debug<T>(mut self, debug_message: T) -> Self
    where
        T: AsRef<str>,
    {
        self.debug = Some(debug_message.as_ref().to_string());
        return self;
    }

    pub fn build(self) -> Error {
        Error {
            message: self.message,
            debug: self.debug,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    message: String,
    debug: Option<String>,
}

impl Error {
    pub fn message(&self) -> &String {
        &self.message
    }
    pub fn debug_message(&self) -> &Option<String> {
        &self.debug
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;
