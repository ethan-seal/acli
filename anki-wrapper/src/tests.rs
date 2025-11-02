use crate::collection::{AnkiCollection, FakeAnkiCollection};
use crate::types::{Card, CardType, DeckConfig};
use crate::error::AnkiWrapperError;

#[test]
fn fake_collection_adds_and_clears_cards() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Test".to_string(),
    };

    col.ensure_deck(&deck).unwrap();

    let c1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Front".into(), "Back".into()],
    };
    col.add_card(&deck, &c1).unwrap();

    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 1);

    col.clear_deck(&deck).unwrap();
    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 0);
}

#[test]
fn fake_collection_create_deck() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "NewDeck".to_string(),
    };

    // Create a new deck
    col.create_deck(&deck).unwrap();
    assert!(col.decks.contains_key(&deck.name));
    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 0);

    // Try to create the same deck again - should error
    let result = col.create_deck(&deck);
    assert!(result.is_err());
    match result {
        Err(AnkiWrapperError::DeckAlreadyExists { name }) => {
            assert_eq!(name, deck.name);
        }
        _ => panic!("Expected DeckAlreadyExists error"),
    }
}

#[test]
fn fake_collection_delete_deck() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "ToDelete".to_string(),
    };

    // Create and add a card to the deck
    col.ensure_deck(&deck).unwrap();
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Q".into(), "A".into()],
    };
    col.add_card(&deck, &card).unwrap();
    assert!(col.decks.contains_key(&deck.name));

    // Delete the deck
    col.delete_deck(&deck).unwrap();
    assert!(!col.decks.contains_key(&deck.name));

    // Try to delete a nonexistent deck - should error
    let result = col.delete_deck(&deck);
    assert!(result.is_err());
    match result {
        Err(AnkiWrapperError::DeckNotFound { name }) => {
            assert_eq!(name, deck.name);
        }
        _ => panic!("Expected DeckNotFound error"),
    }
}

#[test]
fn fake_collection_ensure_vs_create() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "TestDeck".to_string(),
    };

    // ensure_deck should create if doesn't exist
    col.ensure_deck(&deck).unwrap();
    assert!(col.decks.contains_key(&deck.name));

    // ensure_deck on existing deck should succeed
    col.ensure_deck(&deck).unwrap();

    // create_deck on existing deck should fail
    let result = col.create_deck(&deck);
    assert!(result.is_err());
}
