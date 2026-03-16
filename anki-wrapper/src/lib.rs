pub mod collection;
pub mod error;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export core API for convenience
pub use crate::collection::{AnkiCollection, DefaultAnkiCollection, FakeAnkiCollection};
pub use crate::error::AnkiWrapperError;
pub use crate::types::{
    Card, CardId, CardInfo, CardType, DeckConfig, NoteId, ReviewEntry, ReviewRating,
};
