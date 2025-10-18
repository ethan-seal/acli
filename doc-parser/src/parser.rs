//! Parser traits and Markdown parser implementation.
use crate::error::ParseError;
use crate::types::{Card, CardType, ParsedDocument};
use pulldown_cmark::{Event, Parser, Tag};

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
        let mut doc = ParsedDocument::default();

        // Track parent list items' text to build context.
        let mut parents: Vec<String> = Vec::new();
        let mut current_item: Option<String> = None;

        let parser = Parser::new_ext(markdown, pulldown_cmark::Options::empty());
        for event in parser {
            match event {
                Event::Start(Tag::Item) => {
                    current_item = Some(String::new());
                }
                Event::End(Tag::Item) => {
                    if let Some(text) = current_item.take() {
                        let trimmed = text.trim();
                        if let Some((lhs, rhs)) = trimmed.split_once("<->") {
                            let (lhs, rhs) = (lhs.trim(), rhs.trim());
                            let question = build_context_question(&parents, &format!("{} <-> ?", lhs));
                            doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![question, rhs.to_string()] });
                            let rev_q = build_context_question(&parents, &format!("{} <-> ?", rhs));
                            doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![rev_q, lhs.to_string()] });
                        } else if let Some((lhs, rhs)) = trimmed.split_once("->") {
                            let (lhs, rhs) = (lhs.trim(), rhs.trim());
                            let question = build_context_question(&parents, &format!("{} -> ?", lhs));
                            doc.cards.push(Card { card_type: CardType::Basic, fields: vec![question, rhs.to_string()] });
                        } else if !trimmed.is_empty() {
                            // If this list item contains no arrow, treat it as a parent context entry.
                            parents.push(trimmed.to_string());
                        }
                    }
                }
                Event::End(Tag::List(_)) => {
                    // Leaving a list nesting level; drop the last parent if any.
                    if !parents.is_empty() { parents.pop(); }
                }
                Event::Text(t) | Event::Code(t) => {
                    if let Some(buf) = &mut current_item {
                        if !buf.is_empty() { buf.push(' '); }
                        buf.push_str(&t);
                    }
                }
                _ => {}
            }
        }

        // Fallback to line-based parsing if AST yielded nothing
        if doc.cards.is_empty() {
            for (idx, line) in markdown.lines().enumerate() {
                let text = line.trim();
                if text.is_empty() { continue; }
                if let Some((lhs, rhs)) = text.split_once("<->") {
                    let question = format!("{} <-> ?", lhs.trim());
                    doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![question.clone(), rhs.trim().to_string()] });
                    doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![format!("{} <-> ?", rhs.trim()), lhs.trim().to_string()] });
                } else if let Some((lhs, rhs)) = text.split_once("->") {
                    let question = format!("{} -> ?", lhs.trim());
                    doc.cards.push(Card { card_type: CardType::Basic, fields: vec![question, rhs.trim().to_string()] });
                } else if text.starts_with('-') {
                    continue;
                } else {
                    return Err(ParseError::InvalidSyntax { line: idx + 1, column: 1, message: "Unrecognized line format".into() });
                }
            }
        }

        if doc.cards.is_empty() { Err(ParseError::EmptyDocument) } else { Ok(doc) }
    }
}

fn build_context_question(parents: &[String], current: &str) -> String {
    let mut q = String::new();
    for (i, p) in parents.iter().enumerate() {
        let indent = "    ".repeat(i);
        q.push_str(&indent);
        q.push_str("- ");
        q.push_str(p);
        q.push('\n');
    }
    let indent = "    ".repeat(parents.len());
    q.push_str(&indent);
    q.push_str("- ");
    q.push_str(current);
    q
}
