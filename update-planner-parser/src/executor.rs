//! Plan execution traits and default executor.
//!
//! This module intentionally avoids coupling to a concrete Anki API.
//! Consumers can implement `AnkiCollection` for their backend and use
//! `DefaultExecutor` to apply a `SyncPlan`.

use crate::types::{Operation, SyncPlan};

/// Minimal collection interface expected by the executor.
pub trait AnkiCollection {
    /// Ensure a deck with the given name exists.
    fn ensure_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    /// Remove all cards from the deck (fresh sync precondition).
    fn clear_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    /// Add a card to a deck.
    fn add_card(
        &mut self,
        deck_name: &str,
        card: &crate::types::Card,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Delete a note by its ID.
    fn delete_note(
        &mut self,
        deck_name: &str,
        card_id: crate::types::CardId,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Update an existing card by ID.
    fn update_card(
        &mut self,
        deck_name: &str,
        card_id: crate::types::CardId,
        card: &crate::types::Card,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Persist changes.
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Executes a `SyncPlan` against a collection.
pub trait PlanExecutor {
    /// Execute a sync plan against an Anki-like collection.
    fn execute_plan<C: AnkiCollection>(
        &self,
        plan: &SyncPlan,
        collection: &mut C,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Default implementation that performs a fresh sync: clears the deck then adds cards.
pub struct DefaultExecutor;

impl PlanExecutor for DefaultExecutor {
    fn execute_plan<C: AnkiCollection>(
        &self,
        plan: &SyncPlan,
        collection: &mut C,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let deck = &plan.deck_name;
        collection.ensure_deck(deck)?;
        collection.clear_deck(deck)?;

        for op in &plan.operations {
            match op {
                Operation::Add(card) => collection.add_card(deck, card)?,
                Operation::Delete(card_id) => collection.delete_note(deck, *card_id)?,
                Operation::Update(card_id, card) => collection.update_card(deck, *card_id, card)?,
            }
        }

        collection.save()?;
        Ok(())
    }
}
