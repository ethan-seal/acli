#![cfg(feature = "real-anki")]

use anki_wrapper::{AnkiCollection, Card, CardType, DeckConfig, DefaultAnkiCollection};

#[test]
fn real_backend_in_memory_operations() {
    // Create an in-memory collection
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "TestDeck".into(),
    };

    // Test 1: Ensure deck exists
    col.ensure_deck(&deck)
        .expect("Failed to ensure deck exists");

    // Test 2: Add a basic card
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["What is Rust?".into(), "A systems programming language".into()],
    };

    col.add_card(&deck, &card1)
        .expect("Failed to add basic card");

    // Test 3: Add another card
    let card2 = Card {
        card_type: CardType::Basic,
        fields: vec!["What is Anki?".into(), "A spaced repetition system".into()],
    };

    col.add_card(&deck, &card2)
        .expect("Failed to add second card");

    // Test 4: Add a reversed card
    let card3 = Card {
        card_type: CardType::BasicReversed,
        fields: vec!["Front".into(), "Back".into()],
    };

    col.add_card(&deck, &card3)
        .expect("Failed to add reversed card");

    // Test 5: Save the collection
    col.save().expect("Failed to save collection");

    // Test 6: Clear the deck
    col.clear_deck(&deck)
        .expect("Failed to clear deck");

    // Test 7: Verify clearing an already empty deck doesn't error
    col.clear_deck(&deck)
        .expect("Failed to clear already-empty deck");

    // Test 8: Add card after clearing
    col.add_card(&deck, &card1)
        .expect("Failed to add card after clearing");
}

#[test]
fn real_backend_multiple_decks() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck1 = DeckConfig {
        name: "Deck1".into(),
    };
    let deck2 = DeckConfig {
        name: "Deck2".into(),
    };

    col.ensure_deck(&deck1).expect("Failed to create deck1");
    col.ensure_deck(&deck2).expect("Failed to create deck2");

    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Question".into(), "Answer".into()],
    };

    col.add_card(&deck1, &card).expect("Failed to add to deck1");
    col.add_card(&deck2, &card).expect("Failed to add to deck2");

    // Clear only deck1
    col.clear_deck(&deck1).expect("Failed to clear deck1");

    // deck2 should still have cards, but we can't verify that without a read API
    // This test just ensures clear_deck is scoped to the correct deck
}

#[test]
fn real_backend_invalid_card_fields() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "TestDeck".into(),
    };

    col.ensure_deck(&deck).expect("Failed to ensure deck");

    // Card with wrong number of fields
    let invalid_card = Card {
        card_type: CardType::Basic,
        fields: vec!["Only one field".into()],
    };

    let result = col.add_card(&deck, &invalid_card);
    assert!(result.is_err(), "Should fail with wrong number of fields");
}

#[test]
fn real_backend_clear_nonexistent_deck() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "NonexistentDeck".into(),
    };

    // Clearing a deck that doesn't exist should error
    let result = col.clear_deck(&deck);
    assert!(result.is_err(), "Should fail when clearing nonexistent deck");
}

#[test]
fn real_backend_create_deck() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "UniqueTestDeck".into(),
    };

    // Create a new deck
    col.create_deck(&deck)
        .expect("Failed to create new deck");

    // Try to create the same deck again - should error
    let result = col.create_deck(&deck);
    assert!(result.is_err(), "Should fail when creating duplicate deck");

    // Verify we can add cards to it
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Test".into(), "Card".into()],
    };
    col.add_card(&deck, &card)
        .expect("Failed to add card to created deck");
}

#[test]
fn real_backend_delete_deck() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "DeckToDelete".into(),
    };

    // Create and populate a deck
    col.ensure_deck(&deck).expect("Failed to ensure deck");
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Q".into(), "A".into()],
    };
    col.add_card(&deck, &card)
        .expect("Failed to add card");

    // Delete the deck
    col.delete_deck(&deck).expect("Failed to delete deck");

    // Try to delete again - should error
    let result = col.delete_deck(&deck);
    assert!(result.is_err(), "Should fail when deleting nonexistent deck");
}

#[test]
fn real_backend_ensure_vs_create() {
    let mut col = DefaultAnkiCollection::new().expect("Failed to create collection");

    let deck = DeckConfig {
        name: "EnsureVsCreate".into(),
    };

    // ensure_deck should work
    col.ensure_deck(&deck).expect("Failed to ensure deck");

    // ensure_deck again should still work
    col.ensure_deck(&deck)
        .expect("Failed to ensure existing deck");

    // But create_deck should fail
    let result = col.create_deck(&deck);
    assert!(
        result.is_err(),
        "create_deck should fail on existing deck"
    );
}
