use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnkiWrapperError {
    DeckNotFound { name: String },
    InvalidCard { reason: String },
    AnkiError(String),
}

impl Display for AnkiWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnkiWrapperError::DeckNotFound { name } => write!(f, "Deck not found: {}", name),
            AnkiWrapperError::InvalidCard { reason } => write!(f, "Invalid card format: {}", reason),
            AnkiWrapperError::AnkiError(msg) => write!(f, "Anki collection error: {}", msg),
        }
    }
}

impl Error for AnkiWrapperError {}

pub type Result<T> = std::result::Result<T, AnkiWrapperError>;
