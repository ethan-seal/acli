use update_planner_parser::{DocumentDiff, DocumentSet, Operation, types::{Card, CardType}};

#[test]
fn test_diff_empty_to_empty() {
    let old = DocumentSet::default();
    let new = DocumentSet::default();

    let diff = DocumentDiff::compute(&old, &new);

    assert!(diff.added.is_empty());
    assert!(diff.deleted.is_empty());
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_empty_to_nonempty() {
    let old = DocumentSet::default();
    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q2".to_string(), "A2".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);

    assert_eq!(diff.added.len(), 2);
    assert!(diff.deleted.is_empty());
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_nonempty_to_empty() {
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q2".to_string(), "A2".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };
    let new = DocumentSet::default();

    let diff = DocumentDiff::compute(&old, &new);

    assert!(diff.added.is_empty());
    assert_eq!(diff.deleted.len(), 2);
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_same_cards() {
    let cards = vec![
        Card {
            card_type: CardType::Basic,
            fields: vec!["Q1".to_string(), "A1".to_string()],
        },
        Card {
            card_type: CardType::Bidirectional,
            fields: vec!["F".to_string(), "B".to_string()],
        },
    ];

    let old = DocumentSet {
        cards: cards.clone(),
        source_files: vec!["old.md".to_string()],
    };
    let new = DocumentSet {
        cards: cards.clone(),
        source_files: vec!["new.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);

    assert!(diff.added.is_empty());
    assert!(diff.deleted.is_empty());
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_partial_changes() {
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q2".to_string(), "A2".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q3".to_string(), "A3".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };

    let new = DocumentSet {
        cards: vec![
            // Keep Q1/A1 (unchanged)
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            // Q2/A2 is removed (deleted)
            // Q3/A3 is removed (deleted)
            // Add new cards
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q4".to_string(), "A4".to_string()],
            },
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["F".to_string(), "B".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);

    // Two new cards added
    assert_eq!(diff.added.len(), 2);
    // Two old cards deleted
    assert_eq!(diff.deleted.len(), 2);
    // With content-based IDs, updates should be empty
    assert!(diff.updated.is_empty());

    // Verify the added cards are the right ones
    let added_fields: Vec<Vec<String>> = diff.added.iter().map(|c| c.fields.clone()).collect();
    assert!(added_fields.contains(&vec!["Q4".to_string(), "A4".to_string()]));
    assert!(added_fields.contains(&vec!["F".to_string(), "B".to_string()]));

    // Verify the deleted cards are the right ones
    let deleted_fields: Vec<Vec<String>> = diff.deleted.iter().map(|c| c.fields.clone()).collect();
    assert!(deleted_fields.contains(&vec!["Q2".to_string(), "A2".to_string()]));
    assert!(deleted_fields.contains(&vec!["Q3".to_string(), "A3".to_string()]));
}

#[test]
fn test_diff_to_operations() {
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q2".to_string(), "A2".to_string()],
            },
        ],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q1".to_string(), "A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q3".to_string(), "A3".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Should have 1 add and 1 delete
    let add_count = ops.iter().filter(|op| matches!(op, Operation::Add(_))).count();
    let delete_count = ops.iter().filter(|op| matches!(op, Operation::Delete(_))).count();

    assert_eq!(add_count, 1);
    assert_eq!(delete_count, 1);
}

#[test]
fn test_diff_content_change_creates_new_id() {
    // When a card's content changes, with content-based hashing,
    // it should be treated as delete old + add new, not an update
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Question".to_string(), "Answer".to_string()],
            },
        ],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Question".to_string(), "Modified Answer".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);

    // Should be treated as 1 delete + 1 add
    assert_eq!(diff.deleted.len(), 1);
    assert_eq!(diff.added.len(), 1);
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_card_type_change() {
    // Changing card type should create a different ID
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Content".to_string(), "Answer".to_string()],
            },
        ],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Content".to_string(), "Answer".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);

    assert_eq!(diff.deleted.len(), 1);
    assert_eq!(diff.added.len(), 1);
    assert!(diff.updated.is_empty());
}

#[test]
fn test_diff_duplicate_cards() {
    // Test that duplicate cards in the same set are handled correctly
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Q".to_string(), "A".to_string()],
    };

    let old = DocumentSet {
        cards: vec![card.clone(), card.clone()],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![card.clone()],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);

    // Even though there are duplicates, the diff should only report unique cards
    // Since both old cards have the same ID, deleting one means no change from ID perspective
    assert!(diff.added.is_empty());
    assert!(diff.deleted.is_empty());
    assert!(diff.updated.is_empty());
}
