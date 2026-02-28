use doc_parser::{
    Card, CardId, CardType, DocumentParser, MarkdownParser, MediaReference, ParsedDocument,
};
use std::collections::HashSet;

// ── Sequence card tests ────────────────────────────────────────────────────────

#[test]
fn test_sequence_three_steps() {
    let input = "Troubleshoot Wi-Fi connection\n=> check Wi-Fi is enabled\n=> restart device\n=> forget network and reconnect";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 3, "expected 3 sequence cards");

    // Card 1: label / First:  →  first step
    assert_eq!(doc.cards[0].card_type, CardType::Sequence);
    assert_eq!(
        doc.cards[0].fields[0],
        "Troubleshoot Wi-Fi connection\nFirst:"
    );
    assert_eq!(doc.cards[0].fields[1], "check Wi-Fi is enabled");

    // Card 2: label / After: <step1>  →  step 2
    assert_eq!(doc.cards[1].card_type, CardType::Sequence);
    assert_eq!(
        doc.cards[1].fields[0],
        "Troubleshoot Wi-Fi connection\nAfter: check Wi-Fi is enabled"
    );
    assert_eq!(doc.cards[1].fields[1], "restart device");

    // Card 3: label / After: <step2>  →  step 3
    assert_eq!(doc.cards[2].card_type, CardType::Sequence);
    assert_eq!(
        doc.cards[2].fields[0],
        "Troubleshoot Wi-Fi connection\nAfter: restart device"
    );
    assert_eq!(doc.cards[2].fields[1], "forget network and reconnect");
}

#[test]
fn test_sequence_single_step() {
    let input = "Boot a PC\n=> press the power button";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 1);
    assert_eq!(doc.cards[0].card_type, CardType::Sequence);
    assert_eq!(doc.cards[0].fields[0], "Boot a PC\nFirst:");
    assert_eq!(doc.cards[0].fields[1], "press the power button");
}

#[test]
fn test_sequence_no_label_is_ignored() {
    // A `=>` block at the very top of the file (no label above it) should not
    // produce sequence cards, and because nothing else is parseable the result
    // should be an EmptyDocument error.
    let input = "=> step one\n=> step two";
    let parser = MarkdownParser::new();
    let result = parser.parse(input);
    assert!(
        result.is_err(),
        "expected EmptyDocument error when sequence block has no label"
    );
}

#[test]
fn test_sequence_mixed_with_basic_cards() {
    let input = "Reboot steps\n=> power off\n=> wait 10 seconds\n=> power on\n\n- Capital of France? -> Paris";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    // 3 sequence cards + 1 basic card = 4 total
    assert_eq!(doc.cards.len(), 4, "expected 3 sequence + 1 basic cards");

    let seq_count = doc
        .cards
        .iter()
        .filter(|c| c.card_type == CardType::Sequence)
        .count();
    let basic_count = doc
        .cards
        .iter()
        .filter(|c| c.card_type == CardType::Basic)
        .count();

    assert_eq!(seq_count, 3);
    assert_eq!(basic_count, 1);
}

#[test]
fn test_sequence_card_ids_are_deterministic() {
    let input = "Steps\n=> alpha\n=> beta";
    let parser = MarkdownParser::new();
    let doc1 = parser.parse(input).unwrap();
    let doc2 = parser.parse(input).unwrap();

    for (c1, c2) in doc1.cards.iter().zip(doc2.cards.iter()) {
        assert_eq!(c1.id(), c2.id());
    }
}

#[test]
fn test_sequence_card_ids_differ_across_steps() {
    let input = "Steps\n=> alpha\n=> beta";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 2);
    assert_ne!(
        doc.cards[0].id(),
        doc.cards[1].id(),
        "cards in the same sequence must have distinct IDs"
    );
}

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

// ── Attribute card tests (heading + key -> value) ─────────────────────────────

#[test]
fn test_attribute_card_with_heading() {
    let input = "# Hydrogen\n- symbol -> H\n- atomic number -> 1\n- phase -> gas\n";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 3, "expected 3 attribute cards");

    // All cards should be Basic type.
    for card in &doc.cards {
        assert_eq!(card.card_type, CardType::Basic);
    }

    // Front should be two lines: heading on line 1, "key →?" on line 2.
    let symbol_card = doc.cards.iter().find(|c| c.fields[1] == "H").unwrap();
    assert_eq!(symbol_card.fields[0], "Hydrogen\nsymbol \u{2192}?");
    assert_eq!(symbol_card.fields[1], "H");

    let atomic_card = doc.cards.iter().find(|c| c.fields[1] == "1").unwrap();
    assert_eq!(atomic_card.fields[0], "Hydrogen\natomic number \u{2192}?");

    let phase_card = doc.cards.iter().find(|c| c.fields[1] == "gas").unwrap();
    assert_eq!(phase_card.fields[0], "Hydrogen\nphase \u{2192}?");
}

#[test]
fn test_attribute_card_without_heading_is_plain_basic() {
    // No heading → plain basic card, existing format "key -> ?"
    let input = "- symbol -> H\n";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 1);
    let card = &doc.cards[0];
    assert_eq!(card.card_type, CardType::Basic);
    assert_eq!(card.fields[0], "symbol -> ?");
    assert_eq!(card.fields[1], "H");
}

#[test]
fn test_heading_resets_between_sections() {
    let input = "# Hydrogen\n- symbol -> H\n\n# Oxygen\n- symbol -> O\n";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 2);

    let h_card = doc.cards.iter().find(|c| c.fields[1] == "H").unwrap();
    assert_eq!(h_card.fields[0], "Hydrogen\nsymbol \u{2192}?");

    let o_card = doc.cards.iter().find(|c| c.fields[1] == "O").unwrap();
    assert_eq!(o_card.fields[0], "Oxygen\nsymbol \u{2192}?");
}

#[test]
fn test_heading_applies_only_to_basic_not_bidirectional() {
    // <-> items under a heading remain plain bidirectional cards (heading context ignored).
    let input = "# Science\n- hot <-> cold\n";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 1);
    let card = &doc.cards[0];
    assert_eq!(card.card_type, CardType::Bidirectional);
    // Front should NOT contain the heading in a two-line format.
    assert_eq!(card.fields[0], "hot <-> ?");
}

#[test]
fn test_attribute_card_multiple_headings_correct_subject() {
    // Items under H2 should use H2 heading, not H1.
    let input = "# Chemistry\n\n## Hydrogen\n- symbol -> H\n\n## Oxygen\n- symbol -> O\n";
    let parser = MarkdownParser::new();
    let doc = parser.parse(input).unwrap();

    assert_eq!(doc.cards.len(), 2);

    let h_card = doc.cards.iter().find(|c| c.fields[1] == "H").unwrap();
    assert_eq!(h_card.fields[0], "Hydrogen\nsymbol \u{2192}?");

    let o_card = doc.cards.iter().find(|c| c.fields[1] == "O").unwrap();
    assert_eq!(o_card.fields[0], "Oxygen\nsymbol \u{2192}?");
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
