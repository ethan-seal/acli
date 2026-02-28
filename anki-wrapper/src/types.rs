/// Opaque card identifier (wraps i64 from Anki)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CardId(pub i64);

/// Rating given when answering a card during review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewRating {
    Again = 0,
    Hard = 1,
    Good = 2,
    Easy = 3,
}

/// A single review entry, corresponding to one row in Anki's revlog table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewEntry {
    pub card_id: CardId,
    pub rating: ReviewRating,
    /// Interval after this review. Positive = days, negative = seconds (learning steps).
    pub interval: i32,
    /// Ease factor after this review, stored as ease * 1000 (e.g. 2500 = 250%).
    pub ease_factor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    Basic,
    BasicReversed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub card_type: CardType,
    pub fields: Vec<String>,
}

/// Information about a card in the collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardInfo {
    pub id: CardId,
    pub card_type: CardType,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckConfig {
    pub name: String,
}
