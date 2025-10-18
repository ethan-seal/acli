pub mod types;
pub mod error;
pub mod collection;
#[cfg(test)]
mod tests;

// Re-export core API for convenience
pub use crate::types::{Card, CardType, DeckConfig};
pub use crate::collection::{AnkiCollection, DefaultAnkiCollection, FakeAnkiCollection};
pub use crate::error::AnkiWrapperError;
