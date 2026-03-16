/// Opaque card identifier (wraps i64 from Anki)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CardId(pub i64);

/// Opaque note identifier (wraps i64 from Anki).
///
/// In Anki a single *note* produces one or more *cards* — e.g. a
/// "Basic (and reversed card)" note generates two cards.  Deletion
/// and update operations should target notes, not individual cards.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoteId(pub i64);

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

/// Information about a note/card in the collection.
///
/// Each entry represents one Anki *note*.  `id` is the database ID of the
/// first card belonging to the note (useful for review operations), while
/// `note_id` is the note's own ID (used for deletion).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardInfo {
    pub id: CardId,
    pub note_id: NoteId,
    pub card_type: CardType,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckConfig {
    pub name: String,
}
