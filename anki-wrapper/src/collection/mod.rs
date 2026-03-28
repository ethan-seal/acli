use crate::types::{Card, CardId, CardInfo, DeckConfig, NoteId, ReviewEntry, ReviewRating};

pub trait AnkiCollection {
    type Error: std::error::Error + Send + Sync + 'static;

    // === Card / Note Operations ===
    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> std::result::Result<(), Self::Error>;
    fn update_card(&mut self, card_id: CardId, card: &Card)
        -> std::result::Result<(), Self::Error>;
    /// Delete a note (and all its cards) by note ID.
    fn delete_note(&mut self, note_id: NoteId) -> std::result::Result<(), Self::Error>;
    fn get_cards_in_deck(
        &mut self,
        deck: &DeckConfig,
    ) -> std::result::Result<Vec<CardInfo>, Self::Error>;
    fn clear_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;

    // === Deck Operations ===
    fn ensure_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;
    fn create_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;
    fn delete_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;
    fn deck_exists(&mut self, deck: &DeckConfig) -> std::result::Result<bool, Self::Error>;
    fn rename_deck(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> std::result::Result<(), Self::Error>;
    fn get_all_deck_names(&mut self) -> std::result::Result<Vec<String>, Self::Error>;
    fn move_cards_to_deck(
        &mut self,
        card_ids: &[CardId],
        target_deck: &DeckConfig,
    ) -> std::result::Result<(), Self::Error>;

    // === Review Operations ===
    /// Record a review for the given card with the specified rating.
    fn record_review(
        &mut self,
        card_id: CardId,
        rating: ReviewRating,
    ) -> std::result::Result<(), Self::Error>;
    /// Return all recorded reviews for the given card.
    fn get_reviews(
        &mut self,
        card_id: CardId,
    ) -> std::result::Result<Vec<ReviewEntry>, Self::Error>;
}

pub struct DefaultAnkiCollection {
    #[cfg(feature = "real-anki")]
    collection: anki::collection::Collection,
}

mod fake;
#[cfg(not(feature = "real-anki"))]
mod stub;
#[cfg(feature = "real-anki")]
mod real;

pub use fake::FakeAnkiCollection;
