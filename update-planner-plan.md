# update-planner Implementation Plan

## Overview
A simple diff generator that determines what operations (add, delete, update) need to be performed to sync document cards to Anki. Since we're doing fresh syncs initially, this will be simplified.

## Core Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Add(Card),
    Delete(CardId), // For future use when we have card IDs
    Update(CardId, Card), // For future use when we have card IDs
}

#[derive(Debug, Clone)]
pub struct SyncPlan {
    pub operations: Vec<Operation>,
    pub deck_name: String,
}

#[derive(Debug, Clone)]
pub struct DocumentSet {
    pub cards: Vec<Card>,
    pub source_files: Vec<String>, // For tracking which files contributed cards
}
```

## Main API

```rust
pub trait UpdatePlanner {
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Generate sync plan for fresh sync (add all cards, clear deck first)
    fn plan_fresh_sync(&self, documents: &DocumentSet, deck_name: &str) -> Result<SyncPlan, Self::Error>;

    /// Future: Generate incremental sync plan (when we have card IDs)
    fn plan_incremental_sync(
        &self,
        documents: &DocumentSet,
        existing_cards: &[ExistingCard],
        deck_name: &str
    ) -> Result<SyncPlan, Self::Error>;
}

pub struct SimplePlanner;

impl UpdatePlanner for SimplePlanner {
    fn plan_fresh_sync(&self, documents: &DocumentSet, deck_name: &str) -> Result<SyncPlan, Self::Error> {
        // For MVP: just add all cards (anki-wrapper will clear deck first)
        let operations = documents.cards
            .iter()
            .map(|card| Operation::Add(card.clone()))
            .collect();

        Ok(SyncPlan {
            operations,
            deck_name: deck_name.to_string(),
        })
    }
}
```

## Future Card ID System (Placeholder)

```rust
// For when we implement proper card tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardId(String); // Will be UUIDs or content hashes

#[derive(Debug, Clone)]
pub struct ExistingCard {
    pub id: CardId,
    pub card: Card,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct TrackedCard {
    pub id: CardId,
    pub card: Card,
    pub source_file: String,
    pub line_number: usize,
}
```

## Dependencies
- `serde` (optional, for future ID persistence)
- No other dependencies (no-std compatible)

## Error Handling
```rust
#[derive(Debug)]
pub enum PlanError {
    EmptyDocumentSet,
    InvalidDeckName(String),
    DuplicateCards(Vec<Card>), // For future duplicate detection
}

impl core::fmt::Display for PlanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PlanError::EmptyDocumentSet => write!(f, "No cards found in document set"),
            PlanError::InvalidDeckName(name) => write!(f, "Invalid deck name: {}", name),
            PlanError::DuplicateCards(cards) => write!(f, "Duplicate cards found: {} duplicates", cards.len()),
        }
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_sync_plan() {
        let cards = vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Question".to_string(), "Answer".to_string()],
            },
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Front".to_string(), "Back".to_string()],
            },
        ];

        let doc_set = DocumentSet {
            cards,
            source_files: vec!["test.md".to_string()],
        };

        let planner = SimplePlanner;
        let plan = planner.plan_fresh_sync(&doc_set, "TestDeck").unwrap();

        assert_eq!(plan.operations.len(), 2);
        assert_eq!(plan.deck_name, "TestDeck");
        assert!(matches!(plan.operations[0], Operation::Add(_)));
        assert!(matches!(plan.operations[1], Operation::Add(_)));
    }

    #[test]
    fn test_empty_document_set() {
        let doc_set = DocumentSet {
            cards: vec![],
            source_files: vec![],
        };

        let planner = SimplePlanner;
        let result = planner.plan_fresh_sync(&doc_set, "TestDeck");

        assert!(matches!(result, Err(PlanError::EmptyDocumentSet)));
    }
}
```

### Integration Tests
- Test with real parsed documents
- Test various document combinations
- Test error conditions

## Execution Integration

```rust
pub trait PlanExecutor {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Execute a sync plan against an Anki collection
    fn execute_plan<C: AnkiCollection>(
        &self,
        plan: &SyncPlan,
        collection: &mut C
    ) -> Result<(), Self::Error>;
}

pub struct DefaultExecutor;

impl PlanExecutor for DefaultExecutor {
    fn execute_plan<C: AnkiCollection>(
        &self,
        plan: &SyncPlan,
        collection: &mut C
    ) -> Result<(), Self::Error> {
        let deck_config = DeckConfig {
            name: plan.deck_name.clone(),
        };

        // Ensure deck exists
        collection.ensure_deck(&deck_config)?;

        // Clear deck for fresh sync
        collection.clear_deck(&deck_config)?;

        // Execute all operations
        for operation in &plan.operations {
            match operation {
                Operation::Add(card) => {
                    collection.add_card(&deck_config, card)?;
                },
                Operation::Delete(_) => {
                    // Future implementation
                    unimplemented!("Delete operations not yet supported");
                },
                Operation::Update(_, _) => {
                    // Future implementation
                    unimplemented!("Update operations not yet supported");
                },
            }
        }

        collection.save()?;
        Ok(())
    }
}
```

## File Structure
```
update-planner/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── planner.rs      # UpdatePlanner trait + SimplePlanner
│   ├── executor.rs     # PlanExecutor trait + DefaultExecutor
│   ├── types.rs        # Operation, SyncPlan, DocumentSet
│   └── error.rs        # Error types
├── tests/
│   ├── integration.rs  # Integration tests
│   └── unit.rs         # Unit tests
└── examples/
    └── basic_sync.rs   # Example usage
```

## Implementation Steps
1. Define core types (Operation, SyncPlan, DocumentSet)
2. Implement SimplePlanner for fresh syncs
3. Implement DefaultExecutor for plan execution
4. Add comprehensive error handling
5. Write test suite covering all scenarios
6. Add integration with anki-wrapper
7. Documentation and examples
8. Plan for future incremental sync features

## Future Enhancements
1. **Card ID System**: Implement persistent card tracking
2. **Incremental Sync**: Only sync changed cards
3. **Conflict Resolution**: Handle cards modified in both places
4. **Duplicate Detection**: Prevent duplicate cards in same deck
5. **Batch Operations**: Optimize large syncs
6. **Rollback Support**: Undo sync operations if needed

## Performance Considerations
- Minimize memory allocation during plan generation
- Use efficient data structures for card comparison
- Consider streaming for very large document sets
- Batch database operations in executor