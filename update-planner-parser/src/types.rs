/// Card type variants used for planning.
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    /// Basic forward card
    Basic,
    /// Bidirectional card that generates forward and reverse
    Bidirectional,
}

/// Minimal card representation used at planning stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// The card type
    pub card_type: CardType,
    /// Fields such as front/back
    pub fields: Vec<String>,
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

