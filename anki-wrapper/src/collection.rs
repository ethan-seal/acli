use std::collections::HashMap;

use crate::error::{AnkiWrapperError, Result};
use crate::types::{Card, DeckConfig};

pub trait AnkiCollection {
    type Error: std::error::Error + Send + Sync + 'static;

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> std::result::Result<(), Self::Error>;
    fn clear_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;
    fn ensure_deck(&mut self, deck: &DeckConfig) -> std::result::Result<(), Self::Error>;
    fn save(&mut self) -> std::result::Result<(), Self::Error>;
}

pub struct DefaultAnkiCollection {
    #[cfg(feature = "real-anki")]
    _marker: std::marker::PhantomData<()>,
}

impl DefaultAnkiCollection {
    pub fn new() -> Self { Self { #[cfg(feature = "real-anki")] _marker: std::marker::PhantomData } }

    #[cfg(feature = "real-anki")]
    #[allow(dead_code)]
    pub fn open_collection_path<P: AsRef<std::path::Path>>(_path: P) -> Result<Self> {
        // TODO: Wire to anki::collection::Collection when implementing real backend
        Ok(Self { _marker: std::marker::PhantomData })
    }
}

impl AnkiCollection for DefaultAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, _deck: &DeckConfig, _card: &Card) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not wired".to_string(),
        ))
    }

    fn clear_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not wired".to_string(),
        ))
    }

    fn ensure_deck(&mut self, _deck: &DeckConfig) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not wired".to_string(),
        ))
    }

    fn save(&mut self) -> Result<()> {
        Err(AnkiWrapperError::AnkiError(
            "real Anki backend not wired".to_string(),
        ))
    }
}

// Feature-gated real-anki scaffolding for future implementation using the local subrepo.
#[cfg(feature = "real-anki")]
impl DefaultAnkiCollection {
    #[allow(dead_code)]
    pub fn ensure_deck_real(&mut self, _deck: &DeckConfig) -> Result<()> {
        // TODO: use anki APIs to ensure deck exists
        Err(AnkiWrapperError::AnkiError("real Anki ensure_deck not implemented".into()))
    }

    #[allow(dead_code)]
    pub fn add_card_real(&mut self, _deck: &DeckConfig, _card: &Card) -> Result<()> {
        // TODO: create appropriate note type & fields and add to collection
        Err(AnkiWrapperError::AnkiError("real Anki add_card not implemented".into()))
    }

    #[allow(dead_code)]
    pub fn clear_deck_real(&mut self, _deck: &DeckConfig) -> Result<()> {
        // TODO: locate deck and remove its cards
        Err(AnkiWrapperError::AnkiError("real Anki clear_deck not implemented".into()))
    }

    #[allow(dead_code)]
    pub fn save_real(&mut self) -> Result<()> {
        // TODO: persist changes in collection
        Err(AnkiWrapperError::AnkiError("real Anki save not implemented".into()))
    }
}

#[derive(Default)]
pub struct FakeAnkiCollection {
    pub decks: HashMap<String, Vec<Card>>,
    pub save_called: bool,
}

impl FakeAnkiCollection {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AnkiCollection for FakeAnkiCollection {
    type Error = AnkiWrapperError;

    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<()> {
        let entry = self.decks.entry(deck.name.clone()).or_default();
        entry.push(card.clone());
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

    fn save(&mut self) -> Result<()> {
        self.save_called = true;
        Ok(())
    }
}
