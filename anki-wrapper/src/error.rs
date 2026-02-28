use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::types::CardId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnkiWrapperError {
    DeckNotFound { name: String },
    DeckAlreadyExists { name: String },
    CardNotFound { id: CardId },
    InvalidCard { reason: String },
    AnkiError(String),
}

impl Display for AnkiWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnkiWrapperError::DeckNotFound { name } => write!(f, "Deck not found: {}", name),
            AnkiWrapperError::DeckAlreadyExists { name } => {
                write!(f, "Deck already exists: {}", name)
            }
            AnkiWrapperError::CardNotFound { id } => write!(f, "Card not found: {}", id.0),
            AnkiWrapperError::InvalidCard { reason } => {
                write!(f, "Invalid card format: {}", reason)
            }
            AnkiWrapperError::AnkiError(msg) => write!(f, "Anki collection error: {}", msg),
        }
    }
}

impl Error for AnkiWrapperError {}

pub type Result<T> = std::result::Result<T, AnkiWrapperError>;
