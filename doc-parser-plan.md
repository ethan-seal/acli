# doc-parser Implementation Plan

## Overview
A no-std compatible Rust library that parses markdown documents to extract card definitions. Uses a two-pass approach: markdown AST parsing, then custom card extraction.

## Core Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    Basic,       // -> syntax
    Bidirectional, // <-> syntax
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub card_type: CardType,
    pub fields: Vec<String>, // [question, answer]
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub cards: Vec<Card>,
    pub source_path: Option<String>, // For error reporting
}
```

## Main API

```rust
pub trait DocumentParser {
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Parse a markdown string into cards
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error>;
}

pub struct MarkdownParser;

impl DocumentParser for MarkdownParser {
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error> {
        // Implementation
    }
}
```

## Parsing Algorithm

### Step 1: Markdown AST
Use `markdown` crate to parse into AST, focusing on:
- List items (bullet points)
- Text content
- Nested structure

### Step 2: Card Extraction
Walk the AST to find patterns:
- `text -> answer` (Basic card)
- `text <-> answer` (Bidirectional card)
- Preserve all parent context in question field

### Context Preservation Logic
```rust
struct ContextBuilder {
    parents: Vec<String>, // Stack of parent contexts
}

impl ContextBuilder {
    fn build_question(&self, current_text: &str, answer_part: &str) -> String {
        let mut question = String::new();

        // Add all parent context
        for parent in &self.parents {
            question.push_str(parent);
            question.push('\n');
        }

        // Add current context with placeholder
        question.push_str(&current_text.replace(&format!("-> {}", answer_part), "-> ?"));
        question.push_str(&current_text.replace(&format!("<-> {}", answer_part), "<-> ?"));

        question
    }
}
```

## Dependencies
- `markdown` - Markdown parsing (minimal, pure Rust)
- No other dependencies (no-std compatible where possible)

## Error Handling
```rust
#[derive(Debug)]
pub enum ParseError {
    InvalidSyntax { line: usize, column: usize, message: String },
    MarkdownError(String),
    EmptyDocument,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::InvalidSyntax { line, column, message } => {
                write!(f, "Invalid syntax at {}:{}: {}", line, column, message)
            }
            ParseError::MarkdownError(msg) => write!(f, "Markdown parsing error: {}", msg),
            ParseError::EmptyDocument => write!(f, "Document contains no cards"),
        }
    }
}
```

## Testing Strategy

### Unit Tests with Examples
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_card() {
        let input = "One -> 1";
        let expected = Card {
            card_type: CardType::Basic,
            fields: vec!["One -> ?".to_string(), "1".to_string()],
        };

        let parser = MarkdownParser;
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

        let parser = MarkdownParser;
        let result = parser.parse(input).unwrap();
        assert_eq!(result.cards, vec![expected]);
    }

    #[test]
    fn test_nested_context() {
        let input = r#"
- Background material
    - hello -> world
"#;
        let expected = Card {
            card_type: CardType::Basic,
            fields: vec![
                "- Background material\n    - hello -> ?".to_string(),
                "world".to_string()
            ],
        };

        let parser = MarkdownParser;
        let result = parser.parse(input).unwrap();
        assert_eq!(result.cards, vec![expected]);
    }
}
```

### Property Tests
- Roundtrip testing: parse → render → parse should be stable
- Fuzz testing with random markdown input
- Stress testing with deeply nested lists

### Integration Tests
- Test with real markdown files
- Test error conditions (malformed syntax)
- Test edge cases (empty files, no cards, etc.)

## File Structure
```
doc-parser/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── parser.rs       # DocumentParser trait + MarkdownParser
│   ├── types.rs        # Card, CardType, ParsedDocument
│   ├── ast_walker.rs   # AST traversal logic
│   ├── context.rs      # Context preservation logic
│   └── error.rs        # Error types
├── tests/
│   ├── integration.rs  # Real file tests
│   ├── examples.rs     # Test examples from PLAN.md
│   └── property.rs     # Property-based tests
└── benches/
    └── parsing.rs      # Performance benchmarks
```

## Implementation Steps
1. Set up no-std compatible Cargo.toml
2. Define core types (Card, CardType, ParsedDocument)
3. Implement basic markdown parsing with `markdown` crate
4. Create AST walker to find card patterns
5. Implement context preservation logic
6. Add comprehensive error handling
7. Write extensive test suite
8. Add benchmarks for performance
9. Documentation and examples

## Performance Considerations
- Minimize allocations during parsing
- Use string slices where possible
- Consider arena allocation for AST nodes
- Benchmark against realistic document sizes

## Future Extensions
- Support for more card types (cloze, etc.)
- Custom syntax extensions
- Better error reporting with source locations
- Incremental parsing for large documents