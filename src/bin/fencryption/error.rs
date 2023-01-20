pub struct Error {
    error_message: String,
    debug_message: String,
    debug_mode: bool,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!(
            "Error: {}{}",
            self.error_message,
            if self.debug_mode {
                format!("\n{}", self.debug_message)
            } else {
                String::new()
            }
        )
    }
}

pub struct ErrorBuilder {
    error_message: Option<String>,
    debug_message: Option<String>,
    debug_mode: bool,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        ErrorBuilder {
            error_message: None,
            debug_message: None,
            debug_mode: false,
        }
    }

    pub fn error_message<S>(mut self, message: S) -> Self
    where
        S: AsRef<str>,
    {
        self.error_message = Some(message.as_ref().to_string());
        self
    }

    pub fn debug_error<E>(mut self, error: E) -> Self
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

    pub fn debug_mode(mut self, is_debug: bool) -> Self {
        self.debug_mode = is_debug;
        self
    }

    pub fn build(self) -> Error {
        Error {
            error_message: self.error_message.unwrap_or_default(),
            debug_message: self.debug_message.unwrap_or_default(),
            debug_mode: self.debug_mode,
        }
    }
}
