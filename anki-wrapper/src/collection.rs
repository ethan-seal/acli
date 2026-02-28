use std::collections::HashMap;

use crate::error::{AnkiWrapperError, Result};
use crate::types::{Card, CardId, CardInfo, DeckConfig, ReviewEntry, ReviewRating};

pub trait AnkiCollection {
    type Error: std::error::Error + Send + Sync + 'static;

    // === Card Operations ===
    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> std::result::Result<(), Self::Error>;
    fn update_card(&mut self, card_id: CardId, card: &Card)
        -> std::result::Result<(), Self::Error>;
    fn delete_card(&mut self, card_id: CardId) -> std::result::Result<(), Self::Error>;
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

impl DefaultAnkiCollection {
    #[cfg(not(feature = "real-anki"))]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(feature = "real-anki")]
    pub fn new() -> Result<Self> {
        use anki::collection::CollectionBuilder;
        let collection = CollectionBuilder::default().build().map_err(|e| {
            AnkiWrapperError::AnkiError(format!("Failed to create collection: {}", e))
        })?;
        Ok(Self { collection })
    }

    #[cfg(feature = "real-anki")]
    pub fn open_collection_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        use anki::collection::CollectionBuilder;
        let collection = CollectionBuilder::new(path.as_ref()).build().map_err(|e| {
            AnkiWrapperError::AnkiError(format!("Failed to open collection: {}", e))
        })?;
        Ok(Self { collection })
    }
}

#[cfg(not(feature = "real-anki"))]
impl AnkiCollection for DefaultAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, _deck: &DeckConfig, _card: &Card) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn clear_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn ensure_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn create_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn delete_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn update_card(&mut self, _card_id: CardId, _card: &Card) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn delete_card(&mut self, _card_id: CardId) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn get_cards_in_deck(&mut self, _deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn deck_exists(&mut self, _deck: &DeckConfig) -> Result<bool> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn rename_deck(&mut self, _old_name: &str, _new_name: &str) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn get_all_deck_names(&mut self) -> Result<Vec<String>> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn move_cards_to_deck(
        &mut self,
        _card_ids: &[CardId],
        _target_deck: &DeckConfig,
    ) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn record_review(&mut self, _card_id: CardId, _rating: ReviewRating) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }

    fn get_reviews(&mut self, _card_id: CardId) -> Result<Vec<ReviewEntry>> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not compiled (missing 'real-anki' feature)".to_string(),
        ))
    }
}

// Real Anki implementation using rslib
#[cfg(feature = "real-anki")]
impl AnkiCollection for DefaultAnkiCollection {
    type Error = AnkiWrapperError;

    fn ensure_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        self.collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to ensure deck: {}", e)))?;
        Ok(())
    }

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<()> {
        use anki::decks::DeckId;
        use anki::notes::Note;

        // Ensure the deck exists and get its ID
        let deck_obj = self
            .collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(|e| {
                AnkiWrapperError::AnkiError(format!("Failed to get/create deck: {}", e))
            })?;
        let deck_id = DeckId(deck_obj.id.0);

        // Get the appropriate notetype based on card type
        let notetype = match card.card_type {
            crate::types::CardType::Basic => {
                // Get the "Basic" notetype (always exists in new collections)
                let notetypes = self.collection.get_all_notetypes().map_err(|e| {
                    AnkiWrapperError::AnkiError(format!("Failed to get notetypes: {}", e))
                })?;

                // Find the Basic notetype
                notetypes
                    .iter()
                    .find(|nt| nt.name == "Basic")
                    .ok_or_else(|| {
                        AnkiWrapperError::AnkiError("Basic notetype not found".to_string())
                    })?
                    .clone()
            }
            crate::types::CardType::BasicReversed => {
                // Get or create "Basic (and reversed card)" notetype
                let notetypes = self.collection.get_all_notetypes().map_err(|e| {
                    AnkiWrapperError::AnkiError(format!("Failed to get notetypes: {}", e))
                })?;

                // Find the reversed notetype
                notetypes
                    .iter()
                    .find(|nt| nt.name == "Basic (and reversed card)")
                    .ok_or_else(|| {
                        AnkiWrapperError::AnkiError(
                            "Basic (and reversed card) notetype not found".to_string(),
                        )
                    })?
                    .clone()
            }
        };

        // Validate field count
        if card.fields.len() != 2 {
            return Err(AnkiWrapperError::InvalidCard {
                reason: format!("Expected 2 fields, got {}", card.fields.len()),
            });
        }

        // Create a new note with the fields
        let mut note = Note::new(&notetype);
        note.set_field(0, &card.fields[0])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to set field 0: {}", e)))?;
        note.set_field(1, &card.fields[1])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to set field 1: {}", e)))?;

        // Add the note to the collection
        self.collection
            .add_note(&mut note, deck_id)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to add note: {}", e)))?;

        Ok(())
    }

    fn clear_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        use anki::search::SortMode;

        // Verify the deck exists
        let _deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get deck ID: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })?;

        // Search for all cards in the deck
        let search_query = format!("deck:\"{}\"", deck.name);
        let card_ids = self
            .collection
            .search_cards(&search_query, SortMode::NoOrder)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to search cards: {}", e)))?;

        if card_ids.is_empty() {
            return Ok(());
        }

        // Get the note IDs for these cards
        let mut note_ids = Vec::new();
        for card_id in &card_ids {
            if let Some(card) =
                self.collection.storage.get_card(*card_id).map_err(|e| {
                    AnkiWrapperError::AnkiError(format!("Failed to get card: {}", e))
                })?
            {
                note_ids.push(card.note_id());
            }
        }

        // Remove duplicates
        note_ids.sort();
        note_ids.dedup();

        // Remove the notes (this will also remove the cards)
        if !note_ids.is_empty() {
            self.collection.remove_notes(&note_ids).map_err(|e| {
                AnkiWrapperError::AnkiError(format!("Failed to remove notes: {}", e))
            })?;
        }

        Ok(())
    }

    fn create_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        // Check if deck already exists
        let existing_deck_id = self.collection.get_deck_id(&deck.name).map_err(|e| {
            AnkiWrapperError::AnkiError(format!("Failed to check deck existence: {}", e))
        })?;

        if existing_deck_id.is_some() {
            return Err(AnkiWrapperError::DeckAlreadyExists {
                name: deck.name.clone(),
            });
        }

        // Create the deck
        self.collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to create deck: {}", e)))?;

        Ok(())
    }

    fn delete_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        // Get the deck ID
        let deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get deck ID: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })?;

        // Remove the deck and its children (this will also remove all cards in it)
        self.collection
            .remove_decks_and_child_decks(&[deck_id])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to delete deck: {}", e)))?;

        Ok(())
    }

    fn update_card(&mut self, card_id: CardId, card: &Card) -> Result<()> {
        // Get the card to find its note ID
        let anki_card = self
            .collection
            .storage
            .get_card(anki::card::CardId(card_id.0))
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get card: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::CardNotFound { id: card_id })?;

        // Get the note
        let mut note = self
            .collection
            .storage
            .get_note(anki_card.note_id())
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get note: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::AnkiError("Note not found for card".to_string()))?;

        // Validate field count
        if card.fields.len() != 2 {
            return Err(AnkiWrapperError::InvalidCard {
                reason: format!("Expected 2 fields, got {}", card.fields.len()),
            });
        }

        // Update the note fields
        note.set_field(0, &card.fields[0])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to set field 0: {}", e)))?;
        note.set_field(1, &card.fields[1])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to set field 1: {}", e)))?;

        // Update the note in the collection
        self.collection
            .update_note(&mut note)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to update note: {}", e)))?;

        Ok(())
    }

    fn delete_card(&mut self, card_id: CardId) -> Result<()> {
        // Get the card to find its note ID
        let anki_card = self
            .collection
            .storage
            .get_card(anki::card::CardId(card_id.0))
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get card: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::CardNotFound { id: card_id })?;

        // Remove the note (which removes all its cards)
        self.collection
            .remove_notes(&[anki_card.note_id()])
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to remove note: {}", e)))?;

        Ok(())
    }

    fn get_cards_in_deck(&mut self, deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        use anki::search::SortMode;

        // Search for all cards in the deck
        let search_query = format!("deck:\"{}\"", deck.name);
        let card_ids = self
            .collection
            .search_cards(&search_query, SortMode::NoOrder)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to search cards: {}", e)))?;

        let mut result = Vec::new();
        for anki_card_id in card_ids {
            if let Some(anki_card) = self
                .collection
                .storage
                .get_card(anki_card_id)
                .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get card: {}", e)))?
            {
                // Get the note to access fields
                if let Some(note) = self
                    .collection
                    .storage
                    .get_note(anki_card.note_id())
                    .map_err(|e| {
                        AnkiWrapperError::AnkiError(format!("Failed to get note: {}", e))
                    })?
                {
                    // Determine card type based on note type name
                    let notetype = self
                        .collection
                        .get_notetype(note.notetype_id)
                        .map_err(|e| {
                            AnkiWrapperError::AnkiError(format!("Failed to get notetype: {}", e))
                        })?
                        .ok_or_else(|| {
                            AnkiWrapperError::AnkiError("Notetype not found".to_string())
                        })?;

                    let card_type = if notetype.name == "Basic (and reversed card)" {
                        crate::types::CardType::BasicReversed
                    } else {
                        crate::types::CardType::Basic
                    };

                    result.push(CardInfo {
                        id: CardId(anki_card_id.0),
                        card_type,
                        fields: note.fields().clone(),
                    });
                }
            }
        }

        Ok(result)
    }

    fn deck_exists(&mut self, deck: &DeckConfig) -> Result<bool> {
        let deck_id = self.collection.get_deck_id(&deck.name).map_err(|e| {
            AnkiWrapperError::AnkiError(format!("Failed to check deck existence: {}", e))
        })?;
        Ok(deck_id.is_some())
    }

    fn rename_deck(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        // Get the deck ID
        let deck_id = self
            .collection
            .get_deck_id(old_name)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get deck ID: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: old_name.to_string(),
            })?;

        // Get the deck and update its name
        let deck = self
            .collection
            .get_deck(deck_id)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get deck: {}", e)))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: old_name.to_string(),
            })?;

        // Update the name (need to create a mutable copy and manually construct the name)
        let mut deck_mut = (*deck).clone();
        // Parse the new name string into a NativeDeckName
        deck_mut.name = anki::decks::NativeDeckName::from_human_name(new_name);

        // Add or update the deck
        self.collection
            .add_or_update_deck(&mut deck_mut)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to rename deck: {}", e)))?;

        Ok(())
    }

    fn get_all_deck_names(&mut self) -> Result<Vec<String>> {
        let all_deck_names = self
            .collection
            .get_all_deck_names(false)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get all decks: {}", e)))?;

        Ok(all_deck_names.into_iter().map(|(_, name)| name).collect())
    }

    fn move_cards_to_deck(&mut self, card_ids: &[CardId], target_deck: &DeckConfig) -> Result<()> {
        // Ensure target deck exists and get its ID
        let deck_obj = self
            .collection
            .get_or_create_normal_deck(&target_deck.name)
            .map_err(|e| {
                AnkiWrapperError::AnkiError(format!("Failed to get/create target deck: {}", e))
            })?;
        let deck_id = anki::decks::DeckId(deck_obj.id.0);

        // Convert CardId to anki::card::CardId
        let anki_card_ids: Vec<anki::card::CardId> =
            card_ids.iter().map(|id| anki::card::CardId(id.0)).collect();

        // Move cards to the deck
        self.collection
            .set_deck(&anki_card_ids, deck_id)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to move cards: {}", e)))?;

        Ok(())
    }

    fn record_review(&mut self, card_id: CardId, rating: ReviewRating) -> Result<()> {
        let anki_card_id = anki::card::CardId(card_id.0);
        let rating_i32 = rating as i32;
        self.collection
            .grade_now(&[anki_card_id], rating_i32)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to record review: {}", e)))?;
        Ok(())
    }

    fn get_reviews(&mut self, card_id: CardId) -> Result<Vec<ReviewEntry>> {
        let anki_card_id = anki::card::CardId(card_id.0);
        let review_logs = self
            .collection
            .get_review_logs(anki_card_id)
            .map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to get reviews: {}", e)))?;

        let reviews = review_logs
            .entries
            .into_iter()
            .map(|e| {
                let rating = match e.button_chosen {
                    0 => ReviewRating::Again,
                    1 => ReviewRating::Hard,
                    2 => ReviewRating::Good,
                    3 => ReviewRating::Easy,
                    _ => ReviewRating::Again,
                };
                ReviewEntry {
                    card_id,
                    rating,
                    interval: e.interval as i32,
                    ease_factor: e.ease,
                }
            })
            .collect();

        Ok(reviews)
    }
}

pub struct FakeAnkiCollection {
    pub decks: HashMap<String, Vec<CardInfo>>,
    pub reviews: HashMap<CardId, Vec<ReviewEntry>>,
    next_card_id: i64,
}

impl Default for FakeAnkiCollection {
    fn default() -> Self {
        Self {
            decks: HashMap::new(),
            reviews: HashMap::new(),
            next_card_id: 1,
        }
    }
}

impl FakeAnkiCollection {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_card_id(&mut self) -> CardId {
        let id = CardId(self.next_card_id);
        self.next_card_id += 1;
        id
    }
}

impl AnkiCollection for FakeAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<()> {
        let card_id = self.generate_card_id();
        let card_info = CardInfo {
            id: card_id,
            card_type: card.card_type.clone(),
            fields: card.fields.clone(),
        };
        let entry = self.decks.entry(deck.name.clone()).or_default();
        entry.push(card_info);
        Ok(())
    }

    fn clear_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        if let Some(cards) = self.decks.get_mut(&deck.name) {
            cards.clear();
            Ok(())
        } else {
            Err(AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })
        }
    }

    fn ensure_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        self.decks.entry(deck.name.clone()).or_default();
        Ok(())
    }

    fn create_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        if self.decks.contains_key(&deck.name) {
            return Err(AnkiWrapperError::DeckAlreadyExists {
                name: deck.name.clone(),
            });
        }
        self.decks.insert(deck.name.clone(), Vec::new());
        Ok(())
    }

    fn delete_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        if self.decks.remove(&deck.name).is_none() {
            return Err(AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            });
        }
        Ok(())
    }

    fn update_card(&mut self, card_id: CardId, card: &Card) -> Result<()> {
        // Find the card across all decks
        for cards in self.decks.values_mut() {
            if let Some(card_info) = cards.iter_mut().find(|c| c.id == card_id) {
                card_info.card_type = card.card_type.clone();
                card_info.fields = card.fields.clone();
                return Ok(());
            }
        }
        Err(AnkiWrapperError::CardNotFound { id: card_id })
    }

    fn delete_card(&mut self, card_id: CardId) -> Result<()> {
        // Find and remove the card across all decks
        for cards in self.decks.values_mut() {
            if let Some(pos) = cards.iter().position(|c| c.id == card_id) {
                cards.remove(pos);
                return Ok(());
            }
        }
        Err(AnkiWrapperError::CardNotFound { id: card_id })
    }

    fn get_cards_in_deck(&mut self, deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        self.decks
            .get(&deck.name)
            .map(|cards| cards.clone())
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })
    }

    fn deck_exists(&mut self, deck: &DeckConfig) -> Result<bool> {
        Ok(self.decks.contains_key(&deck.name))
    }

    fn rename_deck(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        if !self.decks.contains_key(old_name) {
            return Err(AnkiWrapperError::DeckNotFound {
                name: old_name.to_string(),
            });
        }
        if self.decks.contains_key(new_name) {
            return Err(AnkiWrapperError::DeckAlreadyExists {
                name: new_name.to_string(),
            });
        }
        if let Some(cards) = self.decks.remove(old_name) {
            self.decks.insert(new_name.to_string(), cards);
        }
        Ok(())
    }

    fn get_all_deck_names(&mut self) -> Result<Vec<String>> {
        Ok(self.decks.keys().cloned().collect())
    }

    fn move_cards_to_deck(&mut self, card_ids: &[CardId], target_deck: &DeckConfig) -> Result<()> {
        // Ensure target deck exists
        self.decks.entry(target_deck.name.clone()).or_default();

        // Collect cards to move
        let mut cards_to_move = Vec::new();
        for cards in self.decks.values_mut() {
            cards.retain(|card| {
                if card_ids.contains(&card.id) {
                    cards_to_move.push(card.clone());
                    false
                } else {
                    true
                }
            });
        }

        // Add cards to target deck
        if let Some(target_cards) = self.decks.get_mut(&target_deck.name) {
            target_cards.extend(cards_to_move);
        }

        Ok(())
    }

    fn record_review(&mut self, card_id: CardId, rating: ReviewRating) -> Result<()> {
        // Verify the card exists
        let card_exists = self
            .decks
            .values()
            .any(|cards| cards.iter().any(|c| c.id == card_id));
        if !card_exists {
            return Err(AnkiWrapperError::CardNotFound { id: card_id });
        }
        let entry = ReviewEntry {
            card_id,
            rating,
            // Fake: no real SRS math — interval and ease are left at zero
            interval: 0,
            ease_factor: 0,
        };
        self.reviews.entry(card_id).or_default().push(entry);
        Ok(())
    }

    fn get_reviews(&mut self, card_id: CardId) -> Result<Vec<ReviewEntry>> {
        Ok(self.reviews.get(&card_id).cloned().unwrap_or_default())
    }
}
