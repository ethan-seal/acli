//! Core data types for parsed cards.

/// The kind of card parsed from the document.
#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    /// A one-way card defined with `text -> answer`.
    Basic,
    /// A two-way card defined with `text <-> answer`.
    Bidirectional,
}

/// A single card with its type and fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// The kind of this card (basic or bidirectional).
    pub card_type: CardType,
    /// Card fields, typically `[question, answer]`.
    pub fields: Vec<String>,
}

/// The result of parsing a document, including all extracted cards.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedDocument {
    /// All cards extracted from the document.
    pub cards: Vec<Card>,
    /// Optional source path for diagnostics.
    pub source_path: Option<String>,
}
