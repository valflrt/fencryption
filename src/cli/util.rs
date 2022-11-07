use super::log;

/// Handles command action success
pub fn handle_success(message: Option<String>) {
    if let Some(m) = message {
        log::println_success(m);
    };
    quit::with_code(0);
}

/// Handles command action error
pub fn handle_error(error: ActionError) {
    log::println_error(error.message);
    if let Some(d) = error.debug_message {
        log::println_error(format!("  - {}", d));
    };
    quit::with_code(1);
}

pub struct ActionError {
    message: String,
    debug_message: Option<String>,
}

impl ActionError {
    pub fn new<M>(message: M, debug_message: Option<String>) -> ActionError
    where
        M: AsRef<str>,
    {
        ActionError {
            message: message.as_ref().to_string(),
            debug_message,
        }
    }
}

pub type ActionResult = Result<Option<String>, ActionError>;
