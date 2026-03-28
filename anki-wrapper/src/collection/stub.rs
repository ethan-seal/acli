use crate::error::{AnkiWrapperError, Result};
use crate::types::{Card, CardId, CardInfo, DeckConfig, NoteId, ReviewEntry, ReviewRating};

use super::{AnkiCollection, DefaultAnkiCollection};

/// Error returned by stubs when real Anki backend is not compiled.
fn not_compiled_error<T>() -> Result<T> {
    Err(AnkiWrapperError::AnkiError(
        "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
    ))
}

impl DefaultAnkiCollection {
    pub fn new() -> Self {
        Self {}
    }
}

impl AnkiCollection for DefaultAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, _deck: &DeckConfig, _card: &Card) -> Result<()> {
        not_compiled_error()
    }

    fn clear_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        not_compiled_error()
    }

    fn ensure_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        not_compiled_error()
    }

    fn create_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        not_compiled_error()
    }

    fn delete_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        not_compiled_error()
    }

    fn update_card(&mut self, _card_id: CardId, _card: &Card) -> Result<()> {
        not_compiled_error()
    }

    fn delete_note(&mut self, _note_id: NoteId) -> Result<()> {
        not_compiled_error()
    }

    fn get_cards_in_deck(&mut self, _deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        not_compiled_error()
    }

    fn deck_exists(&mut self, _deck: &DeckConfig) -> Result<bool> {
        not_compiled_error()
    }

    fn rename_deck(&mut self, _old_name: &str, _new_name: &str) -> Result<()> {
        not_compiled_error()
    }

    fn get_all_deck_names(&mut self) -> Result<Vec<String>> {
        not_compiled_error()
    }

    fn move_cards_to_deck(
        &mut self,
        _card_ids: &[CardId],
        _target_deck: &DeckConfig,
    ) -> Result<()> {
        not_compiled_error()
    }

    fn record_review(&mut self, _card_id: CardId, _rating: ReviewRating) -> Result<()> {
        not_compiled_error()
    }

    fn get_reviews(&mut self, _card_id: CardId) -> Result<Vec<ReviewEntry>> {
        not_compiled_error()
    }
}
