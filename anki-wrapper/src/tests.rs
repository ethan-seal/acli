use crate::collection::{AnkiCollection, FakeAnkiCollection};
use crate::error::AnkiWrapperError;
use crate::types::{Card, CardType, DeckConfig, ReviewRating};

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

    // Internal storage has 1 entry (Basic = 1 card per note).
    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 1);

    col.clear_deck(&deck).unwrap();
    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 0);
}

#[test]
fn fake_collection_reversed_note_creates_two_cards() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Test".to_string(),
    };
    col.ensure_deck(&deck).unwrap();

    let card = Card {
        card_type: CardType::BasicReversed,
        fields: vec!["Front".into(), "Back".into()],
    };
    col.add_card(&deck, &card).unwrap();

    // Internal storage: 2 cards for one reversed note.
    assert_eq!(col.decks[&deck.name].len(), 2);
    // Both share the same note_id.
    assert_eq!(
        col.decks[&deck.name][0].note_id,
        col.decks[&deck.name][1].note_id
    );
    // But have different card IDs.
    assert_ne!(col.decks[&deck.name][0].id, col.decks[&deck.name][1].id);

    // get_cards_in_deck deduplicates by note, returning 1 entry.
    let deduped = col.get_cards_in_deck(&deck).unwrap();
    assert_eq!(deduped.len(), 1);
}

#[test]
fn fake_collection_delete_note_removes_all_cards_for_note() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Test".to_string(),
    };
    col.ensure_deck(&deck).unwrap();

    let card = Card {
        card_type: CardType::BasicReversed,
        fields: vec!["Front".into(), "Back".into()],
    };
    col.add_card(&deck, &card).unwrap();
    assert_eq!(col.decks[&deck.name].len(), 2);

    let note_id = col.decks[&deck.name][0].note_id;
    col.delete_note(note_id).unwrap();
    assert_eq!(col.decks[&deck.name].len(), 0);
}

#[test]
fn fake_collection_delete_note_nonexistent() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Test".to_string(),
    };
    col.ensure_deck(&deck).unwrap();

    use crate::types::NoteId;
    let result = col.delete_note(NoteId(999));
    assert!(result.is_err());
    match result {
        Err(AnkiWrapperError::NoteNotFound { id }) => assert_eq!(id, NoteId(999)),
        _ => panic!("expected NoteNotFound error"),
    }
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

#[test]
fn fake_collection_record_and_get_reviews() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "ReviewDeck".to_string(),
    };
    col.ensure_deck(&deck).unwrap();
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Q".into(), "A".into()],
    };
    col.add_card(&deck, &card).unwrap();
    let card_id = col.decks[&deck.name][0].id;

    // No reviews yet
    let reviews = col.get_reviews(card_id).unwrap();
    assert!(
        reviews.is_empty(),
        "expected no reviews before any are recorded"
    );

    // Record a review
    col.record_review(card_id, ReviewRating::Good).unwrap();
    let reviews = col.get_reviews(card_id).unwrap();
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].card_id, card_id);
    assert_eq!(reviews[0].rating, ReviewRating::Good);

    // Record another review with a different rating
    col.record_review(card_id, ReviewRating::Again).unwrap();
    let reviews = col.get_reviews(card_id).unwrap();
    assert_eq!(reviews.len(), 2);
    assert_eq!(reviews[1].rating, ReviewRating::Again);
}

#[test]
fn fake_collection_record_review_requires_existing_card() {
    let mut col = FakeAnkiCollection::new();
    use crate::types::CardId;
    let nonexistent = CardId(999);
    let result = col.record_review(nonexistent, ReviewRating::Good);
    assert!(result.is_err());
    match result {
        Err(AnkiWrapperError::CardNotFound { id }) => assert_eq!(id, nonexistent),
        _ => panic!("expected CardNotFound error"),
    }
}

#[test]
fn fake_collection_reviews_survive_clear_deck() {
    // This test documents the CURRENT (broken) behavior: clear_deck wipes cards,
    // which means reviews for those card IDs are orphaned.
    // Once incremental sync is implemented, this test should be updated to assert
    // that reviews for unchanged cards are preserved across a re-sync.
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Deck".to_string(),
    };
    col.ensure_deck(&deck).unwrap();
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Q".into(), "A".into()],
    };
    col.add_card(&deck, &card).unwrap();
    let card_id = col.decks[&deck.name][0].id;

    // Simulate a review
    col.record_review(card_id, ReviewRating::Good).unwrap();
    assert_eq!(col.get_reviews(card_id).unwrap().len(), 1);

    // clear_deck removes the cards but reviews remain in the HashMap
    col.clear_deck(&deck).unwrap();
    assert_eq!(col.decks[&deck.name].len(), 0, "cards should be cleared");

    // Reviews are still in the map — but the card they belong to no longer exists.
    // This is the data loss that incremental sync must prevent.
    let orphaned_reviews = col.get_reviews(card_id).unwrap();
    assert_eq!(
        orphaned_reviews.len(),
        1,
        "reviews are orphaned (card gone, reviews remain) — incremental sync should prevent this"
    );
}
