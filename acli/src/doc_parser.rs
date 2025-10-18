use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub front: String,
    pub back: String,
}

impl Card {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            front: front.into(),
            back: back.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedDocument {
    pub cards: Vec<Card>,
}

pub trait DocumentParser {
    fn parse(&self, content: &str) -> Result<ParsedDocument, ParseError>;
}

#[derive(Debug, Default, Clone)]
pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentParser for MarkdownParser {
    fn parse(&self, content: &str) -> Result<ParsedDocument, ParseError> {
        let mut cards = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                continue;
            }

            let card_spec = trimmed.trim_start_matches("- ").trim();
            if card_spec.is_empty() {
                return Err(ParseError::invalid_line(
                    line_number + 1,
                    "Card specification cannot be empty",
                ));
            }

            if let Some((front, back)) = split_arrow(card_spec, "<->") {
                cards.push(Card::new(&front, &back));
                cards.push(Card::new(back, front));
            } else if let Some((front, back)) = split_arrow(card_spec, "->") {
                cards.push(Card::new(front, back));
            } else {
                return Err(ParseError::invalid_line(
                    line_number + 1,
                    "Expected '->' or '<->' in card definition",
                ));
            }
        }

        Ok(ParsedDocument { cards })
    }
}

fn split_arrow(text: &str, pattern: &str) -> Option<(String, String)> {
    text.split_once(pattern)
        .map(|(front, back)| (front.trim().to_string(), back.trim().to_string()))
        .filter(|(front, back)| !front.is_empty() && !back.is_empty())
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Line {line}: {message}")]
    InvalidSyntax { line: usize, message: String },
}

impl ParseError {
    pub fn invalid_line(line: usize, message: impl Into<String>) -> Self {
        Self::InvalidSyntax {
            line,
            message: message.into(),
        }
    }
}

impl ParseError {
    pub fn to_string_lossy(&self) -> String {
        format!("{self}")
    }
}
