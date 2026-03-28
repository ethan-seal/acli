use std::collections::HashMap;

use crate::error::{AnkiWrapperError, Result};
use crate::types::{Card, CardId, CardInfo, DeckConfig, NoteId, ReviewEntry, ReviewRating};

use super::AnkiCollection;

pub struct FakeAnkiCollection {
    pub decks: HashMap<String, Vec<CardInfo>>,
    pub reviews: HashMap<CardId, Vec<ReviewEntry>>,
    next_card_id: i64,
    next_note_id: i64,
}

impl Default for FakeAnkiCollection {
    fn default() -> Self {
        Self {
            decks: HashMap::new(),
            reviews: HashMap::new(),
            next_card_id: 1,
            next_note_id: 1,
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

    fn generate_note_id(&mut self) -> NoteId {
        let id = NoteId(self.next_note_id);
        self.next_note_id += 1;
        id
    }
}

impl AnkiCollection for FakeAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<()> {
        let note_id = self.generate_note_id();
        let card_id = self.generate_card_id();

        // BasicReversed notes produce a second (reverse) card, matching
        // real Anki's "Basic (and reversed card)" behaviour.
        let reverse_id = if card.card_type == crate::types::CardType::BasicReversed {
            Some(self.generate_card_id())
        } else {
            None
        };

        let entry = self.decks.entry(deck.name.clone()).or_default();

        // Create the forward card (always).
        entry.push(CardInfo {
            id: card_id,
            note_id,
            card_type: card.card_type.clone(),
            fields: card.fields.clone(),
        });

        if let Some(rev_id) = reverse_id {
            entry.push(CardInfo {
                id: rev_id,
                note_id,
                card_type: card.card_type.clone(),
                fields: card.fields.clone(),
            });
        }

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
        // Find the card to get its note_id, then update all cards sharing
        // that note (mirrors real Anki where updating a note affects all
        // its cards).
        let target_note_id = self
            .decks
            .values()
            .flat_map(|cards| cards.iter())
            .find(|c| c.id == card_id)
            .map(|c| c.note_id)
            .ok_or(AnkiWrapperError::CardNotFound { id: card_id })?;

        for cards in self.decks.values_mut() {
            for card_info in cards.iter_mut().filter(|c| c.note_id == target_note_id) {
                card_info.card_type = card.card_type.clone();
                card_info.fields = card.fields.clone();
            }
        }
        Ok(())
    }

    fn delete_note(&mut self, note_id: NoteId) -> Result<()> {
        // Remove all cards belonging to this note across all decks.
        let mut found = false;
        for cards in self.decks.values_mut() {
            let before = cards.len();
            cards.retain(|c| c.note_id != note_id);
            if cards.len() < before {
                found = true;
            }
        }
        if found {
            Ok(())
        } else {
            Err(AnkiWrapperError::NoteNotFound { id: note_id })
        }
    }

    fn get_cards_in_deck(&mut self, deck: &DeckConfig) -> Result<Vec<CardInfo>> {
        use std::collections::HashSet;

        let cards = self
            .decks
            .get(&deck.name)
            .ok_or_else(|| AnkiWrapperError::DeckNotFound {
                name: deck.name.clone(),
            })?;

        // Deduplicate by note_id — return one entry per note, matching
        // the real Anki implementation.
        let mut seen = HashSet::new();
        Ok(cards
            .iter()
            .filter(|c| seen.insert(c.note_id))
            .cloned()
            .collect())
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
