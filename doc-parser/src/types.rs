//! Core data types for parsed cards.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::error::ParseError;

/// A unique identifier for a card based on its content.
/// This enables tracking cards across syncs even when their position changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(u64);

impl CardId {
    /// Compute a stable card ID from a card's content.
    /// The ID is deterministic: the same card content always produces the same ID.
    pub fn from_card(card: &Card) -> Self {
        let mut hasher = DefaultHasher::new();
        card.hash(&mut hasher);
        CardId(hasher.finish())
    }

    /// Get the raw u64 value of this card ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// The kind of card parsed from the document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardType {
    /// A one-way card defined with `text -> answer`.
    Basic,
    /// A two-way card defined with `text <-> answer`.
    Bidirectional,
    /// A card in an ordered sequence defined with `=> step` blocks.
    /// Fields are `[front, back]` where front is a two-line string.
    Sequence,
}

impl CardType {
    /// Return a short display label for this card type.
    ///
    /// These labels are used in CLI output (e.g. `[basic]`, `[bidi]`).
    pub fn as_label(&self) -> &'static str {
        match self {
            CardType::Basic => "basic",
            CardType::Bidirectional => "bidi",
            CardType::Sequence => "sequence",
        }
    }
}

/// A single card with its type and fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Card {
    /// The kind of this card (basic or bidirectional).
    pub card_type: CardType,
    /// Card fields, typically `[question, answer]`.
    pub fields: Vec<String>,
}

impl Card {
    /// Compute a stable ID for this card based on its content.
    /// The same card content will always produce the same ID.
    pub fn id(&self) -> CardId {
        CardId::from_card(self)
    }
}

/// Characters that are not allowed in an Anki `collection.media` filename.
const INVALID_FILENAME_CHARS: &[char] = &['[', ']', '"', '*', ':', '?', '|', '\\'];

/// A reference to a media file discovered during parsing.
///
/// `source_path` is the path as written in the Markdown (may include
/// directories), while `target_name` is just the filename component that
/// will be stored in Anki's `collection.media`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaReference {
    /// Path as written in the Markdown source, may include directory components.
    pub source_path: String,
    /// Filename component used when storing the file in Anki's collection.media.
    pub target_name: String,
    /// Optional alt-text associated with the image reference.
    pub alt_text: Option<String>,
}

impl MediaReference {
    /// Construct a new `MediaReference`, validating `target_name`.
    ///
    /// Returns `Err(ParseError::InvalidMediaName)` if `source_path` is empty
    /// or if `target_name` contains characters that Anki does not permit
    /// (`[ ] " * : ? | \` or ASCII control characters).
    pub fn new(
        source_path: String,
        target_name: String,
        alt_text: Option<String>,
        line: usize,
    ) -> Result<Self, ParseError> {
        if source_path.is_empty() {
            return Err(ParseError::InvalidMediaName {
                line,
                name: target_name,
                message: "source_path must not be empty".to_string(),
            });
        }

        if let Some(bad) = target_name
            .chars()
            .find(|c| INVALID_FILENAME_CHARS.contains(c) || c.is_ascii_control())
        {
            return Err(ParseError::InvalidMediaName {
                line,
                name: target_name,
                message: format!("contains forbidden character {:?}", bad),
            });
        }

        Ok(Self {
            source_path,
            target_name,
            alt_text,
        })
    }
}

/// A non-fatal warning emitted during parsing.
///
/// Warnings indicate problems that don't prevent card extraction but that the
/// author should fix (e.g. an incomplete block card, a template with no table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// 1-based line number where the warning originated.
    pub line: usize,
    /// Human-readable description of the problem.
    pub message: String,
}

impl Warning {
    /// Create a new warning at the given line.
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// The result of parsing a document, including all extracted cards.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedDocument {
    /// All cards extracted from the document.
    pub cards: Vec<Card>,
    /// All media references discovered in the document.
    pub media: Vec<MediaReference>,
    /// Optional source path for diagnostics.
    pub source_path: Option<String>,
    /// Non-fatal warnings encountered during parsing (e.g. incomplete block cards).
    pub warnings: Vec<Warning>,
}
