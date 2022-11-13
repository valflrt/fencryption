mod decrypt;
mod encrypt;
mod pack;
mod unpack;

pub use decrypt::decrypt;
pub use encrypt::encrypt;
pub use pack::pack;
pub use unpack::unpack;

#[derive(Debug)]
pub struct ActionError {
    message: String,
    debug_message: Option<String>,
}

impl ActionError {
    pub fn new<M>(message: M) -> Self
    where
        M: AsRef<str>,
    {
        ActionError {
            message: message.as_ref().to_string(),
            debug_message: None,
        }
    }

    pub fn new_with_error<M, T>(message: M, error: T) -> Self
    where
        M: AsRef<str>,
        T: std::fmt::Debug,
    {
        ActionError {
            message: message.as_ref().to_string(),
            debug_message: Some(format!("{:#?}", error)),
        }
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn debug_message(&self) -> &Option<String> {
        &self.debug_message
    }
}

pub type ActionResult<T = ()> = Result<T, ActionError>;
