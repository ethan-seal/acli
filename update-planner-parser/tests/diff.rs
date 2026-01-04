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

// ============================================================================
// Integration-level tests: full flow from document states to operations
// ============================================================================

#[test]
fn test_integration_complete_replacement() {
    // Test scenario: replace all cards with completely new ones
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Old Q1".to_string(), "Old A1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Old Q2".to_string(), "Old A2".to_string()],
            },
        ],
        source_files: vec!["old_file.md".to_string()],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["New Front".to_string(), "New Back".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["New Q1".to_string(), "New A1".to_string()],
            },
        ],
        source_files: vec!["new_file.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Should have 2 adds and 2 deletes
    assert_eq!(diff.added.len(), 2);
    assert_eq!(diff.deleted.len(), 2);
    assert!(diff.updated.is_empty());

    let add_count = ops.iter().filter(|op| matches!(op, Operation::Add(_))).count();
    let delete_count = ops.iter().filter(|op| matches!(op, Operation::Delete(_))).count();
    let update_count = ops.iter().filter(|op| matches!(op, Operation::Update(_, _))).count();

    assert_eq!(add_count, 2);
    assert_eq!(delete_count, 2);
    assert_eq!(update_count, 0);
    assert_eq!(ops.len(), 4);
}

#[test]
fn test_integration_operations_from_empty_state() {
    // Starting from empty, add several cards and verify correct operations
    let old = DocumentSet::default();
    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Question 1".to_string(), "Answer 1".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Question 2".to_string(), "Answer 2".to_string()],
            },
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Term".to_string(), "Definition".to_string()],
            },
        ],
        source_files: vec!["notes.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    assert_eq!(ops.len(), 3);
    for op in &ops {
        assert!(matches!(op, Operation::Add(_)));
    }

    // Verify all added cards are present in operations
    let added_cards: Vec<&Card> = ops.iter().filter_map(|op| {
        match op {
            Operation::Add(card) => Some(card),
            _ => None,
        }
    }).collect();

    assert_eq!(added_cards.len(), 3);
    assert!(added_cards.iter().any(|c| c.fields == vec!["Question 1".to_string(), "Answer 1".to_string()]));
    assert!(added_cards.iter().any(|c| c.fields == vec!["Question 2".to_string(), "Answer 2".to_string()]));
    assert!(added_cards.iter().any(|c| c.fields == vec!["Term".to_string(), "Definition".to_string()]));
}

#[test]
fn test_integration_operations_to_empty_state() {
    // Starting from several cards, delete all and verify correct operations
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
                card_type: CardType::Bidirectional,
                fields: vec!["Front".to_string(), "Back".to_string()],
            },
        ],
        source_files: vec!["old.md".to_string()],
    };
    let new = DocumentSet::default();

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    assert_eq!(ops.len(), 3);
    for op in &ops {
        assert!(matches!(op, Operation::Delete(_)));
    }

    // Verify all deleted card IDs correspond to original cards
    let deleted_ids: Vec<_> = ops.iter().filter_map(|op| {
        match op {
            Operation::Delete(id) => Some(id),
            _ => None,
        }
    }).collect();

    assert_eq!(deleted_ids.len(), 3);
    for card in &old.cards {
        assert!(deleted_ids.contains(&&card.id()));
    }
}

#[test]
fn test_integration_mixed_operations() {
    // Complex scenario with simultaneous adds, deletes, and unchanged cards
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Keep This".to_string(), "Unchanged".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["Delete This".to_string(), "Gone".to_string()],
            },
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Also Delete".to_string(), "Removed".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Keep This".to_string(), "Unchanged".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["New Card 1".to_string(), "Added".to_string()],
            },
            Card {
                card_type: CardType::Basic,
                fields: vec!["New Card 2".to_string(), "Also Added".to_string()],
            },
        ],
        source_files: vec!["test.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Should have 2 adds and 2 deletes (1 unchanged card not in operations)
    assert_eq!(diff.added.len(), 2);
    assert_eq!(diff.deleted.len(), 2);
    assert!(diff.updated.is_empty());

    let add_count = ops.iter().filter(|op| matches!(op, Operation::Add(_))).count();
    let delete_count = ops.iter().filter(|op| matches!(op, Operation::Delete(_))).count();

    assert_eq!(add_count, 2);
    assert_eq!(delete_count, 2);
    assert_eq!(ops.len(), 4);
}

#[test]
fn test_integration_reordering_only() {
    // Cards are reordered but content is identical - should produce no operations
    let cards = vec![
        Card {
            card_type: CardType::Basic,
            fields: vec!["First".to_string(), "A".to_string()],
        },
        Card {
            card_type: CardType::Basic,
            fields: vec!["Second".to_string(), "B".to_string()],
        },
        Card {
            card_type: CardType::Basic,
            fields: vec!["Third".to_string(), "C".to_string()],
        },
    ];

    let old = DocumentSet {
        cards: cards.clone(),
        source_files: vec!["test.md".to_string()],
    };

    let new = DocumentSet {
        cards: vec![cards[2].clone(), cards[0].clone(), cards[1].clone()],
        source_files: vec!["test.md".to_string()],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Reordering shouldn't create any operations
    assert!(diff.added.is_empty());
    assert!(diff.deleted.is_empty());
    assert!(diff.updated.is_empty());
    assert_eq!(ops.len(), 0);
}

#[test]
fn test_integration_multiple_field_variations() {
    // Test with cards having different numbers of fields
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q".to_string(), "A".to_string()],
            },
        ],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q".to_string(), "A".to_string(), "Extra".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Different number of fields means different cards
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.deleted.len(), 1);
    assert!(diff.updated.is_empty());
    assert_eq!(ops.len(), 2);
}

#[test]
fn test_integration_whitespace_differences() {
    // Test that whitespace differences are treated as different cards
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
                fields: vec!["Question".to_string(), "Answer ".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Whitespace difference creates different ID
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.deleted.len(), 1);
    assert!(diff.updated.is_empty());
    assert_eq!(ops.len(), 2);
}

#[test]
fn test_integration_large_batch_operations() {
    // Test with a larger number of cards to ensure scalability
    let mut old_cards = Vec::new();
    for i in 0..50 {
        old_cards.push(Card {
            card_type: CardType::Basic,
            fields: vec![format!("Q{}", i), format!("A{}", i)],
        });
    }

    let mut new_cards = Vec::new();
    // Keep first 25 cards unchanged
    for i in 0..25 {
        new_cards.push(Card {
            card_type: CardType::Basic,
            fields: vec![format!("Q{}", i), format!("A{}", i)],
        });
    }
    // Add 25 new cards (replacing the old 25-49)
    for i in 50..75 {
        new_cards.push(Card {
            card_type: CardType::Basic,
            fields: vec![format!("Q{}", i), format!("A{}", i)],
        });
    }

    let old = DocumentSet {
        cards: old_cards,
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: new_cards,
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Should have 25 adds and 25 deletes
    assert_eq!(diff.added.len(), 25);
    assert_eq!(diff.deleted.len(), 25);
    assert!(diff.updated.is_empty());

    let add_count = ops.iter().filter(|op| matches!(op, Operation::Add(_))).count();
    let delete_count = ops.iter().filter(|op| matches!(op, Operation::Delete(_))).count();

    assert_eq!(add_count, 25);
    assert_eq!(delete_count, 25);
    assert_eq!(ops.len(), 50);
}

#[test]
fn test_integration_operation_contains_correct_card_data() {
    // Verify that operations contain the actual card data, not just IDs
    let old = DocumentSet::default();
    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["English".to_string(), "Spanish".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    assert_eq!(ops.len(), 1);
    match &ops[0] {
        Operation::Add(card) => {
            assert_eq!(card.card_type, CardType::Bidirectional);
            assert_eq!(card.fields, vec!["English".to_string(), "Spanish".to_string()]);
        }
        _ => panic!("Expected Add operation"),
    }
}

#[test]
fn test_integration_delete_operation_has_correct_id() {
    // Verify that delete operations contain the correct card ID
    let card = Card {
        card_type: CardType::Basic,
        fields: vec!["Question".to_string(), "Answer".to_string()],
    };
    let expected_id = card.id();

    let old = DocumentSet {
        cards: vec![card],
        source_files: vec![],
    };
    let new = DocumentSet::default();

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    assert_eq!(ops.len(), 1);
    match &ops[0] {
        Operation::Delete(id) => {
            assert_eq!(*id, expected_id);
        }
        _ => panic!("Expected Delete operation"),
    }
}

#[test]
fn test_integration_bidirectional_vs_basic_same_content() {
    // Ensure that same content with different card types produces different operations
    let old = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Front".to_string(), "Back".to_string()],
            },
        ],
        source_files: vec![],
    };

    let new = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Front".to_string(), "Back".to_string()],
            },
        ],
        source_files: vec![],
    };

    let diff = DocumentDiff::compute(&old, &new);
    let ops = diff.to_operations();

    // Different card types should produce different IDs
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.deleted.len(), 1);
    assert!(diff.updated.is_empty());
    assert_eq!(ops.len(), 2);

    // Verify one is Add and one is Delete
    let has_add = ops.iter().any(|op| matches!(op, Operation::Add(_)));
    let has_delete = ops.iter().any(|op| matches!(op, Operation::Delete(_)));
    assert!(has_add);
    assert!(has_delete);
}
