use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid syntax at {line}:{column}: {message}")]
    InvalidSyntax { line: usize, column: usize, message: String },

    #[error("Markdown parsing error: {0}")]
    MarkdownError(String),

    #[error("Document contains no cards")]
    EmptyDocument,
}
