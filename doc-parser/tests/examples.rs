use doc_parser::{Card, CardType, DocumentParser, MarkdownParser};

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
