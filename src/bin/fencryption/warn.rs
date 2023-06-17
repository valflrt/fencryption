#[derive(Debug, Default)]
pub enum WarnKind {
    UnsafeKey,

    #[default]
    Unknown,
}

impl ToString for WarnKind {
    fn to_string(&self) -> String {
        match self {
            WarnKind::UnsafeKey => {
                "Your key should have more than 6 characters to be safer".to_string()
            }
            WarnKind::Unknown => "Unknown warning".to_string(),
        }
    }
}
