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
        let mut doc = ParsedDocument::default();

        // Line-based nested list parser honoring 4-space indents.
        let mut parents: Vec<String> = Vec::new();
        for (idx, raw) in markdown.lines().enumerate() {
            let line = raw.trim_end();
            if line.trim().is_empty() { continue; }

            // Determine indentation depth by counting leading spaces
            let leading_spaces = raw.chars().take_while(|c| *c == ' ').count();
            let is_list_item = line.trim_start().starts_with("- ");
            let depth = if is_list_item { leading_spaces / 4 } else { 0 };
            let content = if is_list_item { line.trim_start().trim_start_matches("- ").trim() } else { line.trim() };

            // Resize parents to current depth
            if parents.len() > depth { parents.truncate(depth); }
            if parents.len() < depth { parents.resize(depth, String::new()); }

            if let Some((lhs, rhs)) = content.split_once("<->") {
                let lhs = lhs.trim();
                let rhs = rhs.trim();
                let question = build_context_question(&parents, &format!("{} <-> ?", lhs));
                doc.cards.push(Card { card_type: CardType::Bidirectional, fields: vec![question, rhs.to_string()] });
            } else if let Some((lhs, rhs)) = content.split_once("->") {
                let lhs = lhs.trim();
                let rhs = rhs.trim();
                let question = build_context_question(&parents, &format!("{} -> ?", lhs));
                doc.cards.push(Card { card_type: CardType::Basic, fields: vec![question, rhs.to_string()] });
            } else if is_list_item {
                // Update parent at this depth
                if parents.len() == depth { parents.push(content.to_string()); } else { parents[depth] = content.to_string(); }
            } else {
                // Non-list line without arrows is invalid
                return Err(ParseError::InvalidSyntax { line: idx + 1, column: 1, message: "Unrecognized line format".into() });
            }
        }

        if doc.cards.is_empty() { Err(ParseError::EmptyDocument) } else { Ok(doc) }
    }
}

fn build_context_question(parents: &[String], current: &str) -> String {
    if parents.is_empty() {
        return current.to_string();
    }
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
