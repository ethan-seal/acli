//! Parser traits and Markdown parser implementation.
use crate::error::ParseError;
use crate::types::{Card, CardType, ParsedDocument};

/// Trait for parsing documents into cards.
pub trait DocumentParser {
    /// Error type returned by the parser.
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Parse a Markdown string into a parsed document containing cards.
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error>;
}

/// Parser that recognizes simple `->` and `<->` card syntax in Markdown.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Create a new Markdown parser instance.
    pub fn new() -> Self { Self }
}

impl Default for MarkdownParser {
    fn default() -> Self { Self::new() }
}

impl DocumentParser for MarkdownParser {
    type Error = ParseError;

    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error> {
        // Temporary simple line-based parser; will replace with AST walker
        let mut doc = ParsedDocument::default();
        for (idx, line) in markdown.lines().enumerate() {
            let text = line.trim();
            if text.is_empty() { continue; }
            if let Some((lhs, rhs)) = text.split_once("<->") {
                let question = format!("{} <-> ?", lhs.trim());
                doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![question, rhs.trim().to_string()] });
            } else if let Some((lhs, rhs)) = text.split_once("->") {
                let question = format!("{} -> ?", lhs.trim());
                doc.cards.push(Card { card_type: CardType::Basic, fields: vec![question, rhs.trim().to_string()] });
            } else if text.starts_with('-') {
                // list item without arrow, ignore for now
                continue;
            } else {
                return Err(ParseError::InvalidSyntax { line: idx + 1, column: 1, message: "Unrecognized line format".into() });
            }
        }
        if doc.cards.is_empty() {
            Err(ParseError::EmptyDocument)
        } else {
            Ok(doc)
        }
    }
}
