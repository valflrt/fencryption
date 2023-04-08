//! Commands.

pub mod decrypt_file;
pub mod decrypt_text;
pub mod encrypt_file;
pub mod encrypt_text;

pub mod logic;

pub enum Command {
    Encrypt,
    Decrypt,
    // Pack,
    // Unpack,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    message: String,
    debug: String,
}

impl Error {
    pub fn to_string(&self, debug_mode: bool) -> String {
        format!(
            "Error: {}{}",
            self.message,
            debug_mode
                .then_some(format!("\n{}", self.debug))
                .unwrap_or("".to_string())
        )
    }
}

pub struct ErrorBuilder {
    message: Option<String>,
    debug: Option<String>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        ErrorBuilder {
            message: None,
            debug: None,
        }
    }

    pub fn message<S>(mut self, message: S) -> Self
    where
        S: AsRef<str>,
    {
        self.message = Some(message.as_ref().to_string());
        self
    }

    pub fn error<E>(mut self, error: E) -> Self
    where
        E: std::fmt::Debug,
    {
        self.debug = Some(format!("{:#?}", error));
        self
    }

    pub fn build(self) -> Error {
        Error {
            message: self.message.unwrap_or_default(),
            debug: self.debug.unwrap_or_default(),
        }
    }
}
