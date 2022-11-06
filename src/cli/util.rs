/// Handles command action success
pub fn handle_success(message: String) {
    println!("\n{}", message);
    quit::with_code(0);
}

/// Handles command action error
pub fn handle_error(error: ActionError) {
    println!("[ERROR] {}", error.message);
    println!("\nAn error occurred: {}", error.message);
    if let Some(d) = error.debug_message {
        println!("  - {:#?}", d)
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

pub type ActionResult = Result<String, ActionError>;
