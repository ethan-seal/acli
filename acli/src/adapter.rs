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

/// Convert plain text to HTML for Anki display.
///
/// Detects markdown-style hierarchical lists (lines starting with "- " with 4-space indentation)
/// and converts them to nested HTML lists. Other text is escaped and newlines become `<br>`.
fn text_to_html(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();

    // Check if this looks like a hierarchical markdown list
    let is_markdown_list = lines.iter().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("- ")
    });

    if !is_markdown_list {
        // Not a markdown list, just escape and convert newlines
        return text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\n', "<br>");
    }

    // Convert markdown list to HTML
    let mut html = String::new();
    let mut depth_stack: Vec<usize> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        // Count leading spaces
        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        let trimmed = line.trim_start();

        if !trimmed.starts_with("- ") {
            continue;
        }

        let depth = leading_spaces / 4;
        let content = trimmed.trim_start_matches("- ");

        // Escape the content
        let escaped_content = content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        // Close lists if we've decreased depth
        while depth_stack.len() > depth + 1 {
            html.push_str("</ul>");
            depth_stack.pop();
        }

        // Open new list if we've increased depth
        if depth_stack.len() == depth {
            html.push_str("<ul>");
            depth_stack.push(depth);
        }

        // Add the list item
        html.push_str("<li>");
        html.push_str(&escaped_content);
        html.push_str("</li>");
    }

    // Close all remaining open lists
    while !depth_stack.is_empty() {
        html.push_str("</ul>");
        depth_stack.pop();
    }

    html
}

/// Convert a planner Card to an anki-wrapper Card.
/// Field content is converted to HTML for proper multi-line display in Anki.
fn convert_card(card: &PlannerCard) -> AnkiWrapperCard {
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
        // Fields are converted to HTML (no change for simple text without newlines)
        assert_eq!(cards[0].fields, vec!["front", "back"]);
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
        // Newlines are converted to <br> for proper Anki display
        assert_eq!(cards[0].fields[0], "line1<br>line2");
        assert_eq!(cards[0].fields[1], "answer");
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
        // HTML special characters are escaped
        assert_eq!(cards[0].fields[0], "x &lt; y &amp; y &gt; z");
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

        let cards = &adapter.inner().decks["Test"];
        assert_eq!(cards.len(), 1);

        // The front field should be converted to nested HTML lists
        let expected_html = "<ul><li>Spanish</li><ul><li>Greetings</li><ul><li>Hello &lt;-&gt; ?</li></ul></ul></ul>";
        assert_eq!(cards[0].fields[0], expected_html);
        assert_eq!(cards[0].fields[1], "Hola");
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
