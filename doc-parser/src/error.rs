//! Error types for the document parser.
use thiserror::Error;

/// Errors that can occur while parsing a document.
#[derive(Debug, Error)]
pub enum ParseError {
    /// A syntactic error was detected at a given line and column.
    #[error("Invalid syntax at {line}:{column}: {message}")]
    InvalidSyntax {
        /// 1-based line number where the error occurred.
        line: usize,
        /// 1-based column number where the error occurred.
        column: usize,
        /// Human-readable description of the problem.
        message: String,
    },

    /// An error occurred while parsing Markdown input.
    #[error("Markdown parsing error: {0}")]
    MarkdownError(String),

    /// The document contained no recognizable cards.
    #[error("Document contains no cards")]
    EmptyDocument,

    /// A media reference contained an invalid `target_name`.
    #[error("Invalid media filename at line {line}: \"{name}\" — {message}. Consider renaming the file.")]
    InvalidMediaName {
        /// 1-based line number where the image reference appears.
        line: usize,
        /// The invalid filename that was provided.
        name: String,
        /// Human-readable description of what's wrong.
        message: String,
    },

    /// An error in a template block (empty cell, unknown placeholder, etc.).
    #[error("Template error: {0}")]
    TemplateError(String),
}
