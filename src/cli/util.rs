use super::log;

/// Handles command action error
pub fn handle_error(error: CommandError) {
    log::println_error(error.message);
    if let Some(d) = error.debug_message {
        log::println_error(log::with_start_line(d, "    "));
    };
    quit::with_code(1);
}

pub struct CommandError {
    message: String,
    debug_message: Option<String>,
}

impl CommandError {
    pub fn new<M>(message: M, debug_message: Option<String>) -> Self
    where
        M: AsRef<str>,
    {
        CommandError {
            message: message.as_ref().to_string(),
            debug_message,
        }
    }
}

pub type CommandResult<T = ()> = Result<T, CommandError>;
