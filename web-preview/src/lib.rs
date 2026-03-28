//! Lightweight web preview server for acli flashcards.
//!
//! Serves a self-contained HTML page showing parsed cards grouped by source file.
//! The `refresh` closure is called on every page load so changes are picked up
//! with a simple browser refresh — no file watcher needed.

mod renderer;
mod server;
mod static_files;
mod url_rewriting;

// Re-export public API
pub use server::serve;
pub use url_rewriting::rewrite_image_urls;

// ── Public types ─────────────────────────────────────────────────────────────

/// Card type for preview display.
#[derive(Debug, Clone)]
pub enum CardType {
    Basic,
    Bidirectional,
    Sequence,
}

impl CardType {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            CardType::Basic => "Basic",
            CardType::Bidirectional => "Bidirectional",
            CardType::Sequence => "Sequence",
        }
    }

    pub(crate) fn css_class(&self) -> &'static str {
        match self {
            CardType::Basic => "basic",
            CardType::Bidirectional => "bidi",
            CardType::Sequence => "sequence",
        }
    }
}

/// A card to display in the web preview.
#[derive(Debug, Clone)]
pub struct PreviewCard {
    /// Raw text for the front of the card (will be rendered as markdown).
    pub front: String,
    /// Raw text for the back of the card (will be rendered as markdown).
    pub back: String,
    /// Card type.
    pub card_type: CardType,
    /// Source file path (for grouping).
    pub source_file: Option<String>,
}

/// Data for rendering a preview page.
#[derive(Debug, Clone)]
pub struct PreviewData {
    /// Deck name for display.
    pub deck_name: String,
    /// All cards to display.
    pub cards: Vec<PreviewCard>,
    /// Number of markdown files scanned.
    pub files_processed: usize,
    /// Non-fatal warnings or per-file parse errors.
    pub errors: Vec<String>,
}
