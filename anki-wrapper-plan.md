# anki-wrapper Implementation Plan

## Overview
A minimal Rust library that wraps Anki's core functionality for creating, updating, and deleting cards. This will be a separate crate in the `anki-wrapper/` subdirectory.

## Core Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    Basic,
    BasicReversed, // Maps to Anki's "Basic (and reversed card)"
}

#[derive(Debug, Clone)]
pub struct Card {
    pub card_type: CardType,
    pub fields: Vec<String>, // [front, back] for Basic, [front, back] for BasicReversed
}

#[derive(Debug)]
pub struct DeckConfig {
    pub name: String,
}
```

## Main Trait

```rust
pub trait AnkiCollection {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Add a card to the specified deck
    fn add_card(&mut self, deck: &DeckConfig, card: &Card) -> Result<(), Self::Error>;

    /// Clear all cards from the specified deck (for fresh sync)
    fn clear_deck(&mut self, deck: &DeckConfig) -> Result<(), Self::Error>;

    /// Ensure deck exists, create if necessary
    fn ensure_deck(&mut self, deck: &DeckConfig) -> Result<(), Self::Error>;

    /// Save changes to the collection
    fn save(&mut self) -> Result<(), Self::Error>;
}
```

## Implementation Structure

```rust
pub struct DefaultAnkiCollection {
    // Will contain Anki collection instance from subrepo
}

impl AnkiCollection for DefaultAnkiCollection {
    // Implementation using Anki's Rust APIs
}
```

## Dependencies
- `anki` (from git subrepo) - Core Anki functionality
- `thiserror` - Error handling (minimal)
- No other dependencies to keep it lightweight

## Testing Strategy

### Unit Tests (with Fakes)
```rust
pub struct FakeAnkiCollection {
    pub decks: HashMap<String, Vec<Card>>,
    pub save_called: bool,
}

impl AnkiCollection for FakeAnkiCollection {
    // In-memory implementation for testing
}
```

### Integration Tests
- Test against real Anki collection file
- Verify cards are actually created in Anki format
- Test deck creation and management
- Test error conditions (invalid deck names, etc.)

## Error Handling
```rust
#[derive(thiserror::Error, Debug)]
pub enum AnkiWrapperError {
    #[error("Deck not found: {name}")]
    DeckNotFound { name: String },

    #[error("Invalid card format: {reason}")]
    InvalidCard { reason: String },

    #[error("Anki collection error: {0}")]
    AnkiError(#[from] anki::error::AnkiError),
}
```

## API Design Goals
- Minimal surface area
- Clear ownership (no lifetimes if possible)
- Easy to mock/fake for testing
- Fail fast on errors
- No async (keep it simple)

## File Structure
```
anki-wrapper/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── collection.rs   # AnkiCollection trait + DefaultAnkiCollection
│   ├── types.rs        # Card, CardType, DeckConfig
│   ├── error.rs        # Error types
│   └── fake.rs         # FakeAnkiCollection for testing
├── tests/
│   ├── integration.rs  # Real Anki collection tests
│   └── unit.rs         # Unit tests with fakes
└── anki/               # Git subrepo (will be added)
```

## Implementation Steps
1. Set up Cargo.toml with minimal dependencies
2. Define core types (Card, CardType, DeckConfig)
3. Define AnkiCollection trait
4. Implement FakeAnkiCollection for testing
5. Set up integration test framework
6. Implement DefaultAnkiCollection using Anki APIs
7. Add comprehensive error handling
8. Add documentation and examples