//! Core data types for parsed cards.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A unique identifier for a card based on its content.
/// This enables tracking cards across syncs even when their position changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(u64);

impl CardId {
    /// Compute a stable card ID from a card's content.
    /// The ID is deterministic: the same card content always produces the same ID.
    pub fn from_card(card: &Card) -> Self {
        let mut hasher = DefaultHasher::new();
        card.hash(&mut hasher);
        CardId(hasher.finish())
    }

    /// Get the raw u64 value of this card ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// The kind of card parsed from the document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardType {
    /// A one-way card defined with `text -> answer`.
    Basic,
    /// A two-way card defined with `text <-> answer`.
    Bidirectional,
}

/// A single card with its type and fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    /// The kind of this card (basic or bidirectional).
    pub card_type: CardType,
    /// Card fields, typically `[question, answer]`.
    pub fields: Vec<String>,
}

impl Card {
    /// Compute a stable ID for this card based on its content.
    /// The same card content will always produce the same ID.
    pub fn id(&self) -> CardId {
        CardId::from_card(self)
    }
}

/// The result of parsing a document, including all extracted cards.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedDocument {
    /// All cards extracted from the document.
    pub cards: Vec<Card>,
    /// Optional source path for diagnostics.
    pub source_path: Option<String>,
}
