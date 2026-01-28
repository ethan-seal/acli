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
    /// Delete a card by its ID
    Delete(CardId),
    /// Update an existing card by ID
    Update(CardId, Card),
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

/// Result of comparing old vs new document state.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentDiff {
    /// Cards that exist in new but not in old (to be added)
    pub added: Vec<Card>,
    /// Cards that exist in old but not in new (to be deleted)
    pub deleted: Vec<Card>,
    /// Cards with same ID but different content (to be updated)
    /// Note: With content-based hashing, this will be empty since
    /// any content change produces a different CardId. This is here
    /// for future extensibility if we switch to position-based IDs.
    pub updated: Vec<(Card, Card)>, // (old, new)
}

impl DocumentDiff {
    /// Compute the diff between an old and new document set.
    pub fn compute(old: &DocumentSet, new: &DocumentSet) -> Self {
        use std::collections::{HashMap, HashSet};

        // Build maps of CardId -> Card for fast lookup
        let old_map: HashMap<CardId, &Card> = old.cards.iter().map(|c| (c.id(), c)).collect();
        let new_map: HashMap<CardId, &Card> = new.cards.iter().map(|c| (c.id(), c)).collect();

        let old_ids: HashSet<CardId> = old_map.keys().copied().collect();
        let new_ids: HashSet<CardId> = new_map.keys().copied().collect();

        // Cards in new but not in old = added
        let added: Vec<Card> = new_ids
            .difference(&old_ids)
            .filter_map(|id| new_map.get(id))
            .map(|&c| c.clone())
            .collect();

        // Cards in old but not in new = deleted
        let deleted: Vec<Card> = old_ids
            .difference(&new_ids)
            .filter_map(|id| old_map.get(id))
            .map(|&c| c.clone())
            .collect();

        // Cards with same ID but different content = updated
        // With content-based hashing, this should always be empty
        // since any change in content creates a new CardId
        let updated: Vec<(Card, Card)> = old_ids
            .intersection(&new_ids)
            .filter_map(|id| {
                let old_card = old_map.get(id)?;
                let new_card = new_map.get(id)?;
                // If CardIds are the same but cards differ (shouldn't happen with current impl)
                if old_card != new_card {
                    Some(((*old_card).clone(), (*new_card).clone()))
                } else {
                    None
                }
            })
            .collect();

        DocumentDiff {
            added,
            deleted,
            updated,
        }
    }

    /// Convert this diff into a list of operations.
    pub fn to_operations(&self) -> Vec<Operation> {
        let mut ops = Vec::new();

        // Add all new cards
        for card in &self.added {
            ops.push(Operation::Add(card.clone()));
        }

        // Delete removed cards
        for card in &self.deleted {
            ops.push(Operation::Delete(card.id()));
        }

        // Update modified cards (should be empty with content-based IDs)
        for (_old, new) in &self.updated {
            ops.push(Operation::Update(new.id(), new.clone()));
        }

        ops
    }
}
