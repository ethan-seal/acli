//! Adapter to bridge anki-wrapper types with update-planner-parser's AnkiCollection trait.
//!
//! This module provides `AnkiCollectionAdapter<C>` which wraps any `anki_wrapper::AnkiCollection`
//! implementation and implements `update_planner_parser::executor::AnkiCollection`.

use anki_wrapper::{
    AnkiCollection as AnkiWrapperCollection, Card as AnkiWrapperCard,
    CardType as AnkiWrapperCardType, DeckConfig, FakeAnkiCollection,
};
use update_planner_parser::executor::AnkiCollection as ExecutorCollection;
use update_planner_parser::types::{
    Card as PlannerCard, CardId as PlannerCardId, CardType as PlannerCardType,
};

/// Adapter that wraps an anki_wrapper::AnkiCollection and implements
/// update_planner_parser::executor::AnkiCollection.
pub struct AnkiCollectionAdapter<C> {
    inner: C,
}

impl<C> AnkiCollectionAdapter<C> {
    /// Create a new adapter wrapping the given collection.
    pub fn new(collection: C) -> Self {
        Self { inner: collection }
    }

    /// Get a reference to the inner collection.
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Get a mutable reference to the inner collection.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }

    /// Consume the adapter and return the inner collection.
    pub fn into_inner(self) -> C {
        self.inner
    }
}

impl AnkiCollectionAdapter<FakeAnkiCollection> {
    /// Create a new adapter with a fake in-memory collection (for testing).
    pub fn fake() -> Self {
        Self::new(FakeAnkiCollection::new())
    }
}

/// Convert a planner Card to an anki-wrapper Card.
fn convert_card(card: &PlannerCard) -> AnkiWrapperCard {
    let card_type = match card.card_type {
        PlannerCardType::Basic => AnkiWrapperCardType::Basic,
        PlannerCardType::Bidirectional => AnkiWrapperCardType::BasicReversed,
    };
    AnkiWrapperCard {
        card_type,
        fields: card.fields.clone(),
    }
}

impl<C> ExecutorCollection for AnkiCollectionAdapter<C>
where
    C: AnkiWrapperCollection,
{
    fn ensure_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        self.inner
            .ensure_deck(&deck)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn clear_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        self.inner
            .clear_deck(&deck)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn add_card(
        &mut self,
        deck_name: &str,
        card: &PlannerCard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        let anki_card = convert_card(card);
        self.inner
            .add_card(&deck, &anki_card)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn delete_card(
        &mut self,
        _deck_name: &str,
        card_id: PlannerCardId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // anki-wrapper's delete_card doesn't need deck name
        // We need to convert the content-hash based PlannerCardId to anki-wrapper's i64 CardId
        // For now, we can't do this directly since the IDs are fundamentally different.
        // The planner's CardId is a content hash (u64), while anki-wrapper's is database ID (i64).
        //
        // In a fresh sync scenario (which DefaultExecutor uses), we clear the deck first,
        // so delete operations don't actually occur. If we need true incremental sync,
        // we'd need to maintain a mapping between content hashes and database IDs.
        //
        // For now, return an error if delete is called (shouldn't happen with fresh sync).
        Err(format!(
            "delete_card not supported: content-hash CardId {} cannot be mapped to database ID",
            card_id
        )
        .into())
    }

    fn update_card(
        &mut self,
        _deck_name: &str,
        card_id: PlannerCardId,
        _card: &PlannerCard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Same issue as delete_card - we can't map content-hash IDs to database IDs
        Err(format!(
            "update_card not supported: content-hash CardId {} cannot be mapped to database ID",
            card_id
        )
        .into())
    }

    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // anki-wrapper doesn't have an explicit save method
        // Operations are committed immediately in the real Anki backend
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use update_planner_parser::{DefaultExecutor, Operation, PlanExecutor, SyncPlan};

    #[test]
    fn test_adapter_ensure_deck() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test::Deck").unwrap();

        // Verify deck was created
        assert!(adapter.inner().decks.contains_key("Test::Deck"));
    }

    #[test]
    fn test_adapter_add_card() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test").unwrap();

        let card = PlannerCard {
            card_type: PlannerCardType::Basic,
            fields: vec!["front".to_string(), "back".to_string()],
        };
        adapter.add_card("Test", &card).unwrap();

        let cards = &adapter.inner().decks["Test"];
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].fields, vec!["front", "back"]);
    }

    #[test]
    fn test_adapter_clear_deck() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test").unwrap();

        let card = PlannerCard {
            card_type: PlannerCardType::Basic,
            fields: vec!["front".to_string(), "back".to_string()],
        };
        adapter.add_card("Test", &card).unwrap();
        assert_eq!(adapter.inner().decks["Test"].len(), 1);

        adapter.clear_deck("Test").unwrap();
        assert_eq!(adapter.inner().decks["Test"].len(), 0);
    }

    #[test]
    fn test_adapter_with_executor() {
        let mut adapter = AnkiCollectionAdapter::fake();
        let executor = DefaultExecutor;

        let plan = SyncPlan {
            deck_name: "MyDeck".to_string(),
            operations: vec![
                Operation::Add(PlannerCard {
                    card_type: PlannerCardType::Basic,
                    fields: vec!["Q1".to_string(), "A1".to_string()],
                }),
                Operation::Add(PlannerCard {
                    card_type: PlannerCardType::Bidirectional,
                    fields: vec!["Q2".to_string(), "A2".to_string()],
                }),
            ],
        };

        executor.execute_plan(&plan, &mut adapter).unwrap();

        let cards = &adapter.inner().decks["MyDeck"];
        assert_eq!(cards.len(), 2);
    }
}
