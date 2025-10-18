use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnkiWrapperError {
    #[error("Deck not found: {name}")]
    DeckNotFound { name: String },

    #[error("Invalid card format: {reason}")]
    InvalidCard { reason: String },

    // Placeholder for future mapping to real Anki errors.
    #[error("Anki collection error: {0}")]
    AnkiError(String),
}

pub type Result<T> = std::result::Result<T, AnkiWrapperError>;
