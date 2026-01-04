//! Core data types used by the update planner.

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

/// Card type variants used for planning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardType {
    /// Basic forward card
    Basic,
    /// Bidirectional card that generates forward and reverse
    Bidirectional,
}

/// Minimal card representation used at planning stage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    /// The card type
    pub card_type: CardType,
    /// Fields such as front/back
    pub fields: Vec<String>,
}

impl Card {
    /// Compute a stable ID for this card based on its content.
    /// The same card content will always produce the same ID.
    pub fn id(&self) -> CardId {
        CardId::from_card(self)
    }
}

/// Planned operation to apply to a collection.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Add a new card
    Add(Card),
    /// Delete by identifier (future)
    Delete(String),
    /// Update existing card (future)
    Update(String, Card),
}

/// A set of parsed documents with extracted cards.
#[derive(Debug, Clone, Default)]
pub struct DocumentSet {
    /// All extracted cards
    pub cards: Vec<Card>,
    /// Source file paths
    pub source_files: Vec<String>,
}

/// A sync plan describing what to execute against Anki.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncPlan {
    /// Operations to perform
    pub operations: Vec<Operation>,
    /// Target deck name
    pub deck_name: String,
}
