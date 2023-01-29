#[derive(Debug)]
pub struct Error {
    error_message: String,
    debug_message: String,
}

impl Error {
    pub fn to_string(&self, debug_mode: bool) -> String {
        format!(
            "Error: {}{}",
            self.error_message,
            debug_mode
                .then_some(format!("\n{}", self.debug_message))
                .unwrap_or("".to_string())
        )
    }
}

pub struct ErrorBuilder {
    error_message: Option<String>,
    debug_message: Option<String>,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        ErrorBuilder {
            error_message: None,
            debug_message: None,
        }
    }

    pub fn message<S>(mut self, message: S) -> Self
    where
        S: AsRef<str>,
    {
        self.error_message = Some(message.as_ref().to_string());
        self
    }

    pub fn error<E>(mut self, error: E) -> Self
    where
        E: std::fmt::Debug,
    {
        self.debug_message = Some(format!("{:#?}", error));
        self
    }

    pub fn debug_message<S>(mut self, message: S) -> Self
    where
        S: AsRef<str>,
    {
        self.debug_message = Some(message.as_ref().to_string());
        self
    }

    pub fn build(self) -> Error {
        Error {
            error_message: self.error_message.unwrap_or_default(),
            debug_message: self.debug_message.unwrap_or_default(),
        }
    }
}
