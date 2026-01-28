use std::collections::HashSet;
use update_planner_parser::{
    types::{Card, CardType},
    CardId,
};

#[test]
fn test_card_id_deterministic() {
    // Same card content should always produce the same ID
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Question".to_string(), "Answer".to_string()],
    };
    let card2 = Card {
        card_type: CardType::Basic,
        fields: vec!["Question".to_string(), "Answer".to_string()],
    };

    assert_eq!(card1.id(), card2.id());
    assert_eq!(CardId::from_card(&card1), CardId::from_card(&card2));
}

#[test]
fn test_card_id_different_content() {
    // Different card content should produce different IDs
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Question 1".to_string(), "Answer 1".to_string()],
    };
    let card2 = Card {
        card_type: CardType::Basic,
        fields: vec!["Question 2".to_string(), "Answer 2".to_string()],
    };

    assert_ne!(card1.id(), card2.id());
}

#[test]
fn test_card_id_different_types() {
    // Same fields but different types should produce different IDs
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Content".to_string(), "Answer".to_string()],
    };
    let card2 = Card {
        card_type: CardType::Bidirectional,
        fields: vec!["Content".to_string(), "Answer".to_string()],
    };

    assert_ne!(card1.id(), card2.id());
}

#[test]
fn test_card_id_usable_in_collections() {
    // CardIds should be usable as keys in hash-based collections
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Q1".to_string(), "A1".to_string()],
    };
    let card2 = Card {
        card_type: CardType::Basic,
        fields: vec!["Q2".to_string(), "A2".to_string()],
    };
    let card3 = Card {
        card_type: CardType::Basic,
        fields: vec!["Q1".to_string(), "A1".to_string()],
    };

    let mut seen_ids = HashSet::new();
    seen_ids.insert(card1.id());
    seen_ids.insert(card2.id());
    seen_ids.insert(card3.id());

    // card1 and card3 have the same content, so should have same ID
    assert_eq!(seen_ids.len(), 2);
    assert!(seen_ids.contains(&card1.id()));
    assert!(seen_ids.contains(&card2.id()));
}

#[test]
fn test_card_id_display() {
    // CardId should have a readable hex display format
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Test".to_string(), "Card".to_string()],
    };
    let id = card.id();
    let display = format!("{}", id);

    // Should be 16 hex digits
    assert_eq!(display.len(), 16);
    assert!(display.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_card_id_ordering() {
    // CardIds should be orderable for use in sorted collections
    let card1 = Card {
        card_type: CardType::Basic,
        fields: vec!["A".to_string(), "1".to_string()],
    };
    let card2 = Card {
        card_type: CardType::Basic,
        fields: vec!["B".to_string(), "2".to_string()],
    };
    let card3 = Card {
        card_type: CardType::Basic,
        fields: vec!["C".to_string(), "3".to_string()],
    };

    let mut ids = vec![card2.id(), card1.id(), card3.id()];
    ids.sort();

    // Just verify that sorting works (we don't care about the specific order)
    assert_eq!(ids.len(), 3);
}
