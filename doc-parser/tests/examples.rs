use doc_parser::{
    Card, CardId, CardType, DocumentParser, MarkdownParser, MediaReference, ParsedDocument,
};
use std::collections::HashSet;

#[test]
fn test_basic_card() {
    let input = "One -> 1";
    let expected = Card {
        card_type: CardType::Basic,
        fields: vec!["One -> ?".to_string(), "1".to_string()],
    };

    let parser = MarkdownParser::new();
    let result = parser.parse(input).unwrap();
    assert_eq!(result.cards, vec![expected]);
}

#[test]
fn test_bidirectional_card() {
    let input = "One <-> 1";
    let expected = Card {
        card_type: CardType::Bidirectional,
        fields: vec!["One <-> ?".to_string(), "1".to_string()],
    };

    let parser = MarkdownParser::new();
    let result = parser.parse(input).unwrap();
    assert_eq!(result.cards, vec![expected]);
}

#[test]
fn test_nested_context_list() {
    let input = r#"
- Background material
    - hello -> world
"#;

    let expected = Card {
        card_type: CardType::Basic,
        fields: vec![
            "- Background material\n    - hello -> ?".to_string(),
            "world".to_string(),
        ],
    };

    let parser = MarkdownParser::new();
    let result = parser.parse(input).unwrap();
    assert!(result.cards.contains(&expected));
}

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

// ── MediaReference tests ───────────────────────────────────────────────────────

#[test]
fn test_media_reference_construction() {
    let mr = MediaReference::new(
        "images/cat.jpg".to_string(),
        "cat.jpg".to_string(),
        Some("a cat".to_string()),
        1,
    )
    .expect("valid MediaReference should construct without error");

    assert_eq!(mr.source_path, "images/cat.jpg");
    assert_eq!(mr.target_name, "cat.jpg");
    assert_eq!(mr.alt_text, Some("a cat".to_string()));
}

#[test]
fn test_media_reference_no_alt() {
    let mr = MediaReference::new("photo.png".to_string(), "photo.png".to_string(), None, 5)
        .expect("valid MediaReference with no alt should construct");

    assert_eq!(mr.alt_text, None);
}

#[test]
fn test_parsed_document_default_has_empty_media() {
    let doc = ParsedDocument::default();
    assert!(
        doc.media.is_empty(),
        "default ParsedDocument.media must be empty"
    );
}

#[test]
fn test_media_reference_rejects_square_brackets() {
    let err = MediaReference::new(
        "image[1].jpg".to_string(),
        "image[1].jpg".to_string(),
        None,
        3,
    )
    .expect_err("target_name with '[' should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("image[1].jpg"),
        "error should mention the bad filename: {msg}"
    );
}

#[test]
fn test_media_reference_rejects_double_quote() {
    let err = MediaReference::new(
        "file\"name.png".to_string(),
        "file\"name.png".to_string(),
        None,
        7,
    )
    .expect_err("target_name with '\"' should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("file\"name.png"),
        "error should mention the bad filename: {msg}"
    );
}

#[test]
fn test_media_reference_rejects_colon() {
    let err = MediaReference::new("img:2.jpg".to_string(), "img:2.jpg".to_string(), None, 2)
        .expect_err("target_name with ':' should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("img:2.jpg"),
        "error should mention the bad filename: {msg}"
    );
}

#[test]
fn test_media_reference_rejects_empty_source_path() {
    let err = MediaReference::new(String::new(), "valid.jpg".to_string(), None, 1)
        .expect_err("empty source_path should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("source_path must not be empty"),
        "error should mention empty source: {msg}"
    );
}

#[test]
fn test_media_reference_rejects_control_characters() {
    // target_name containing a null byte (control character)
    let err = MediaReference::new("file\0.jpg".to_string(), "file\0.jpg".to_string(), None, 4)
        .expect_err("target_name with control character should be rejected");
    assert!(!err.to_string().is_empty());
}
