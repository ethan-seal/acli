#![deny(missing_docs)]
//! Document parser library for extracting card definitions from Markdown.

pub mod error;
pub mod parser;
pub mod types;

pub use crate::error::ParseError;
pub use crate::parser::{DocumentParser, MarkdownParser};
pub use crate::types::{Card, CardId, CardType, ParsedDocument};
