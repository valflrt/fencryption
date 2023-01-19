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

    pub fn error<T>(mut self, error: T) -> Self
    where
        T: std::fmt::Debug,
    {
        self.debug_message = Some(format!("{:#?}", error));
        return self;
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn debug_message(&self) -> &Option<String> {
        &self.debug_message
    }
}

pub type ActionResult<T = ()> = Result<T, ActionError>;
