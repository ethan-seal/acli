//! Adapter to bridge anki-wrapper types with update-planner-parser's AnkiCollection trait.
//!
//! This module provides `AnkiCollectionAdapter<C>` which wraps any `anki_wrapper::AnkiCollection`
//! implementation and implements `update_planner_parser::executor::AnkiCollection`.

use anki_wrapper::{
    AnkiCollection as AnkiWrapperCollection, Card as AnkiWrapperCard, CardInfo,
    CardType as AnkiWrapperCardType, DeckConfig, FakeAnkiCollection, NoteId,
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

impl<C: AnkiWrapperCollection> AnkiCollectionAdapter<C> {
    /// Ensure a deck exists, creating it if needed.
    pub fn ensure_deck(&mut self, deck_name: &str) -> Result<(), C::Error> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        self.inner.ensure_deck(&deck)
    }

    /// Get all cards currently in the given deck.
    /// Returns an empty vec if the deck doesn't exist.
    pub fn get_cards_in_deck(&mut self, deck_name: &str) -> Result<Vec<CardInfo>, C::Error> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        match self.inner.get_cards_in_deck(&deck) {
            Ok(cards) => Ok(cards),
            Err(e) => {
                // If deck doesn't exist, return empty — caller will create it
                let err_str = format!("{}", e);
                if err_str.contains("not found") {
                    Ok(Vec::new())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Delete a note (and all its cards) by Anki note ID.
    pub fn delete_note_by_id(&mut self, note_id: NoteId) -> Result<(), C::Error> {
        self.inner.delete_note(note_id)
    }

    /// Add a card to a deck. Converts the planner card to HTML for Anki display.
    pub fn add_card_to_deck(
        &mut self,
        deck_name: &str,
        card: &PlannerCard,
    ) -> Result<(), C::Error> {
        let deck = DeckConfig {
            name: deck_name.to_string(),
        };
        let anki_card = convert_card(card);
        self.inner.add_card(&deck, &anki_card)
    }

    /// Save/persist changes.
    pub fn save(&mut self) -> Result<(), C::Error> {
        // Real Anki backend commits immediately; fake has nothing to do.
        // This is a no-op but keeps the API consistent.
        Ok(())
    }
}

impl AnkiCollectionAdapter<FakeAnkiCollection> {
    /// Create a new adapter with a fake in-memory collection (for testing).
    pub fn fake() -> Self {
        Self::new(FakeAnkiCollection::new())
    }
}

/// Convert card field text to HTML for Anki display.
/// Public so that sync logic can convert parsed cards to HTML for comparison.
///
/// Uses pulldown-cmark to render full markdown: bullet lists, images, code
/// spans, emphasis, etc.  Single newlines are converted to hard breaks so
/// they produce visible `<br>` in the output, matching the behaviour users
/// expect when writing multiline card content.
pub fn text_to_html(text: &str) -> String {
    // Convert single newlines to markdown hard breaks (two trailing spaces
    // before the newline) so pulldown-cmark emits <br /> for each line break.
    let with_hard_breaks = text.replace('\n', "  \n");

    let parser = pulldown_cmark::Parser::new(&with_hard_breaks);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    // Trim trailing whitespace/newlines that pulldown-cmark may add.
    let trimmed = html.trim_end();
    trimmed.to_string()
}

/// Convert a planner Card to an anki-wrapper Card.
/// Field content is converted to HTML for proper multi-line display in Anki.
pub fn convert_card(card: &PlannerCard) -> AnkiWrapperCard {
    let card_type = match card.card_type {
        PlannerCardType::Basic => AnkiWrapperCardType::Basic,
        PlannerCardType::Bidirectional => AnkiWrapperCardType::BasicReversed,
        PlannerCardType::Sequence => AnkiWrapperCardType::Basic,
    };
    AnkiWrapperCard {
        card_type,
        fields: card.fields.iter().map(|f| text_to_html(f)).collect(),
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

    fn delete_note(
        &mut self,
        _deck_name: &str,
        card_id: PlannerCardId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // We can't map the content-hash based PlannerCardId to anki-wrapper's
        // NoteId.  The planner's CardId is a content hash (u64), while
        // anki-wrapper's NoteId is a database ID (i64).
        //
        // In a fresh sync scenario (which DefaultExecutor uses), we clear the
        // deck first, so delete operations don't actually occur.  The real
        // incremental sync path in sync.rs works with NoteIds directly.
        Err(format!(
            "delete_note not supported: content-hash CardId {} cannot be mapped to database NoteId",
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
        // Fields are rendered as HTML via pulldown-cmark (wrapped in <p> tags)
        assert_eq!(cards[0].fields, vec!["<p>front</p>", "<p>back</p>"]);
    }

    #[test]
    fn test_adapter_converts_newlines_to_html() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test").unwrap();

        let card = PlannerCard {
            card_type: PlannerCardType::Basic,
            fields: vec!["line1\nline2".to_string(), "answer".to_string()],
        };
        adapter.add_card("Test", &card).unwrap();

        let cards = &adapter.inner().decks["Test"];
        assert_eq!(cards.len(), 1);
        // Newlines become hard breaks via pulldown-cmark
        assert_eq!(cards[0].fields[0], "<p>line1<br />\nline2</p>");
        assert_eq!(cards[0].fields[1], "<p>answer</p>");
    }

    #[test]
    fn test_adapter_escapes_html_special_chars() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test").unwrap();

        let card = PlannerCard {
            card_type: PlannerCardType::Basic,
            fields: vec!["x < y & y > z".to_string(), "true".to_string()],
        };
        adapter.add_card("Test", &card).unwrap();

        let cards = &adapter.inner().decks["Test"];
        assert_eq!(cards.len(), 1);
        // HTML special characters are escaped (wrapped in <p> by pulldown-cmark)
        assert_eq!(cards[0].fields[0], "<p>x &lt; y &amp; y &gt; z</p>");
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
    fn test_adapter_converts_markdown_list_to_html() {
        let mut adapter = AnkiCollectionAdapter::fake();
        adapter.ensure_deck("Test").unwrap();

        let hierarchical_front = "- Spanish\n    - Greetings\n        - Hello <-> ?";
        let card = PlannerCard {
            card_type: PlannerCardType::Bidirectional,
            fields: vec![hierarchical_front.to_string(), "Hola".to_string()],
        };
        adapter.add_card("Test", &card).unwrap();

        // get_cards_in_deck deduplicates by note — 1 reversed note = 1 entry.
        let cards = adapter.get_cards_in_deck("Test").unwrap();
        assert_eq!(cards.len(), 1);

        // The front field should be a nested HTML list (via pulldown-cmark)
        let front = &cards[0].fields[0];
        assert!(front.contains("<ul>"), "should contain <ul> tags: {front}");
        assert!(front.contains("<li>"), "should contain <li> tags: {front}");
        assert!(
            front.contains("Spanish"),
            "should contain 'Spanish': {front}"
        );
        assert!(
            front.contains("Greetings"),
            "should contain 'Greetings': {front}"
        );
        assert!(
            front.contains("Hello &lt;-&gt; ?"),
            "should contain escaped arrow: {front}"
        );
        assert_eq!(cards[0].fields[1], "<p>Hola</p>");
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

        // get_cards_in_deck returns 2 notes (Basic + Bidirectional).
        // Internally there are 3 cards (1 Basic + 2 for the reversed note),
        // but the public API deduplicates by note.
        let cards = adapter.get_cards_in_deck("MyDeck").unwrap();
        assert_eq!(cards.len(), 2);
    }
}
