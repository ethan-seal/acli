use crate::types::{Card, Warning};

/// Result of a pre-processing extraction pass.
///
/// Each extractor (pipe table, sequence, block) returns this type so
/// the orchestrator doesn't have to handle different tuple shapes.
pub(super) struct ExtractionResult {
    pub cards: Vec<Card>,
    pub residual: String,
    pub warnings: Vec<Warning>,
}

/// Build residual text from lines not marked as consumed.
///
/// This is the shared implementation of the consumed-line reconstruction
/// pattern used by all three extractors.
pub(super) fn build_residual(lines: &[&str], consumed: &[bool]) -> String {
    lines
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed[*idx])
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n")
}
