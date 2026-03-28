use crate::error::{AnkiWrapperError, Result};
use crate::types::{Card, CardId, CardInfo, DeckConfig, NoteId, ReviewEntry, ReviewRating};

use super::{AnkiCollection, DefaultAnkiCollection};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build an `AnkiWrapperError::AnkiError` from an operation description and an error value.
///
/// Returns a closure suitable for use with `.map_err(anki_err("Failed to X"))`.
fn anki_err(
    operation: &'static str,
) -> impl Fn(impl std::fmt::Display) -> AnkiWrapperError {
    move |e| AnkiWrapperError::AnkiError(format!("{}: {}", operation, e))
}

/// Find the appropriate notetype for a given card type.
fn find_notetype(
    collection: &mut anki::collection::Collection,
    card_type: &crate::types::CardType,
) -> Result<anki::notetype::Notetype> {
    let notetypes = collection
        .get_all_notetypes()
        .map_err(anki_err("Failed to get notetypes"))?;

    let notetype_name = match card_type {
        crate::types::CardType::Basic => "Basic",
        crate::types::CardType::BasicReversed => "Basic (and reversed card)",
    };

    notetypes
        .iter()
        .find(|nt| nt.name == notetype_name)
        .ok_or_else(|| {
            AnkiWrapperError::AnkiError(format!("{} notetype not found", notetype_name))
        })
        .cloned()
}

/// Set the fields of a note, validating field count.
///
/// Both Basic and BasicReversed card types use exactly 2 fields.
fn set_note_fields(note: &mut anki::notes::Note, fields: &[String]) -> Result<()> {
    if fields.len() != 2 {
        return Err(AnkiWrapperError::InvalidCard {
            reason: format!("Expected 2 fields, got {}", fields.len()),
        });
    }
    note.set_field(0, &fields[0])
        .map_err(anki_err("Failed to set field 0"))?;
    note.set_field(1, &fields[1])
        .map_err(anki_err("Failed to set field 1"))?;
    Ok(())
}

// ── Constructor ───────────────────────────────────────────────────────────────

impl DefaultAnkiCollection {
    pub fn new() -> Result<Self> {
        use anki::collection::CollectionBuilder;
        let collection = CollectionBuilder::default().build().map_err(|e| {
            AnkiWrapperError::AnkiError(format!("Failed to create collection: {}", e))
        })?;
        Ok(Self { collection })
    }

    pub fn open_collection_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        use anki::collection::CollectionBuilder;
        let collection = CollectionBuilder::new(path.as_ref()).build().map_err(|e| {
            let msg = format!("{}", e);
            if msg.contains("locked") || msg.contains("busy") {
                AnkiWrapperError::AnkiError(
                    "Failed to open collection: the database is locked.\n\
                     This usually means Anki is currently running. \
                     Close Anki and try again."
                        .to_string(),
                )
            } else {
                AnkiWrapperError::AnkiError(format!("Failed to open collection: {}", e))
            }
        })?;
        Ok(Self { collection })
    }
}

// ── AnkiCollection impl ───────────────────────────────────────────────────────

impl AnkiCollection for DefaultAnkiCollection {
    type Error = AnkiWrapperError;

    fn ensure_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        self.collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(anki_err("Failed to ensure deck"))?;
        Ok(())
    }

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<()> {
        use anki::decks::DeckId;
        use anki::notes::Note;

        // Ensure the deck exists and get its ID
        let deck_obj = self
            .collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(anki_err("Failed to get/create deck"))?;
        let deck_id = DeckId(deck_obj.id.0);

        // Get the appropriate notetype based on card type
        let notetype = find_notetype(&mut self.collection, &card.card_type)?;

        // Create a new note and set its fields
        let mut note = Note::new(&notetype);
        set_note_fields(&mut note, &card.fields)?;

        // Add the note to the collection
        self.collection
            .add_note(&mut note, deck_id)
            .map_err(anki_err("Failed to add note"))?;

        Ok(())
    }

    fn clear_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        use anki::search::SortMode;

        // Verify the deck exists
        let _deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(anki_err("Failed to get deck ID"))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })?;

        // Search for all cards in the deck
        let search_query = format!("deck:\"{}\"", deck.name);
        let card_ids = self
            .collection
            .search_cards(&search_query, SortMode::NoOrder)
            .map_err(anki_err("Failed to search cards"))?;

        if card_ids.is_empty() {
            return Ok(());
        }

        // Get the note IDs for these cards
        let mut note_ids = Vec::new();
        for card_id in &card_ids {
            if let Some(card) =
                self.collection.storage.get_card(*card_id).map_err(anki_err("Failed to get card"))?
            {
                note_ids.push(card.note_id());
            }
        }

        // Remove duplicates
        note_ids.sort();
        note_ids.dedup();

        // Remove the notes (this will also remove the cards)
        if !note_ids.is_empty() {
            self.collection.remove_notes(&note_ids).map_err(anki_err("Failed to remove notes"))?;
        }

        Ok(())
    }

    fn create_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        // Check if deck already exists
        let existing_deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(anki_err("Failed to check deck existence"))?;

        if existing_deck_id.is_some() {
            return Err(AnkiWrapperError::DeckAlreadyExists {
                name: deck.name.clone(),
            });
        }

        // Create the deck
        self.collection
            .get_or_create_normal_deck(&deck.name)
            .map_err(anki_err("Failed to create deck"))?;

        Ok(())
    }

    fn delete_deck(&mut self, deck: &DeckConfig) -> Result<()> {
        // Get the deck ID
        let deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(anki_err("Failed to get deck ID"))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })?;

        // Remove the deck and its children (this will also remove all cards in it)
        self.collection
            .remove_decks_and_child_decks(&[deck_id])
            .map_err(anki_err("Failed to delete deck"))?;

        Ok(())
    }

    fn update_card(&mut self, card_id: CardId, card: &Card) -> Result<()> {
        // Get the card to find its note ID
        let anki_card = self
            .collection
            .storage
            .get_card(anki::card::CardId(card_id.0))
            .map_err(anki_err("Failed to get card"))?
            .ok_or_else(|| AnkiWrapperError::CardNotFound { id: card_id })?;

        // Get the note
        let mut note = self
            .collection
            .storage
            .get_note(anki_card.note_id())
            .map_err(anki_err("Failed to get note"))?
            .ok_or_else(|| AnkiWrapperError::AnkiError("Note not found for card".to_string()))?;

        // Update the note fields
        set_note_fields(&mut note, &card.fields)?;

        // Update the note in the collection
        self.collection
            .update_note(&mut note)
            .map_err(anki_err("Failed to update note"))?;

        Ok(())
    }

    fn delete_note(&mut self, note_id: NoteId) -> Result<()> {
        let anki_note_id = anki::notes::NoteId(note_id.0);

        // Verify the note exists before attempting removal.
        let _note = self
            .collection
            .storage
            .get_note(anki_note_id)
            .map_err(anki_err("Failed to get note"))?
            .ok_or_else(|| AnkiWrapperError::NoteNotFound { id: note_id })?;

        // Remove the note (and all its cards).
        self.collection
            .remove_notes(&[anki_note_id])
            .map_err(anki_err("Failed to remove note"))?;

        Ok(())
    }

    fn get_cards_in_deck(&mut self, deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        use anki::search::SortMode;
        use std::collections::HashSet;

        // Search for all cards in the deck
        let search_query = format!("deck:\"{}\"", deck.name);
        let card_ids = self
            .collection
            .search_cards(&search_query, SortMode::NoOrder)
            .map_err(anki_err("Failed to search cards"))?;

        let mut result = Vec::new();
        let mut seen_notes = HashSet::new();

        for anki_card_id in card_ids {
            let anki_card = match self
                .collection
                .storage
                .get_card(anki_card_id)
                .map_err(anki_err("Failed to get card"))?
            {
                Some(c) => c,
                None => continue,
            };

            // Deduplicate by note: only emit one CardInfo per note.
            // A "Basic (and reversed card)" note produces two Anki cards
            // but represents a single user-authored card.
            if !seen_notes.insert(anki_card.note_id()) {
                continue;
            }

            let note = match self
                .collection
                .storage
                .get_note(anki_card.note_id())
                .map_err(anki_err("Failed to get note"))?
            {
                Some(n) => n,
                None => continue,
            };

            // Determine card type based on note type name
            let notetype = self
                .collection
                .get_notetype(note.notetype_id)
                .map_err(anki_err("Failed to get notetype"))?
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
                note_id: NoteId(anki_card.note_id().0),
                card_type,
                fields: note.fields().clone(),
            });
        }

        Ok(result)
    }

    fn deck_exists(&mut self, deck: &DeckConfig) -> Result<bool> {
        let deck_id = self
            .collection
            .get_deck_id(&deck.name)
            .map_err(anki_err("Failed to check deck existence"))?;
        Ok(deck_id.is_some())
    }

    fn rename_deck(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        // Get the deck ID
        let deck_id = self
            .collection
            .get_deck_id(old_name)
            .map_err(anki_err("Failed to get deck ID"))?
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: old_name.to_string(),
            })?;

        // Get the deck and update its name
        let deck = self
            .collection
            .get_deck(deck_id)
            .map_err(anki_err("Failed to get deck"))?
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
            .map_err(anki_err("Failed to rename deck"))?;

        Ok(())
    }

    fn get_all_deck_names(&mut self) -> Result<Vec<String>> {
        let all_deck_names = self
            .collection
            .get_all_deck_names(false)
            .map_err(anki_err("Failed to get all decks"))?;

        Ok(all_deck_names.into_iter().map(|(_, name)| name).collect())
    }

    fn move_cards_to_deck(&mut self, card_ids: &[CardId], target_deck: &DeckConfig) -> Result<()> {
        // Ensure target deck exists and get its ID
        let deck_obj = self
            .collection
            .get_or_create_normal_deck(&target_deck.name)
            .map_err(anki_err("Failed to get/create target deck"))?;
        let deck_id = anki::decks::DeckId(deck_obj.id.0);

        // Convert CardId to anki::card::CardId
        let anki_card_ids: Vec<anki::card::CardId> =
            card_ids.iter().map(|id| anki::card::CardId(id.0)).collect();

        // Move cards to the deck
        self.collection
            .set_deck(&anki_card_ids, deck_id)
            .map_err(anki_err("Failed to move cards"))?;

        Ok(())
    }

    fn record_review(&mut self, card_id: CardId, rating: ReviewRating) -> Result<()> {
        let anki_card_id = anki::card::CardId(card_id.0);
        let rating_i32 = rating as i32;
        self.collection
            .grade_now(&[anki_card_id], rating_i32)
            .map_err(anki_err("Failed to record review"))?;
        Ok(())
    }

    fn get_reviews(&mut self, card_id: CardId) -> Result<Vec<ReviewEntry>> {
        let anki_card_id = anki::card::CardId(card_id.0);
        let review_logs = self
            .collection
            .get_review_logs(anki_card_id)
            .map_err(anki_err("Failed to get reviews"))?;

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
