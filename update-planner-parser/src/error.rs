//! Error types for update planning.
use core::fmt;
use std::error::Error;

/// Planning errors produced when generating sync plans.
#[derive(Debug)]
pub enum PlanError {
    /// No cards found to plan against
    EmptyDocumentSet,
    /// Deck name is invalid
    InvalidDeckName(String),
    /// Placeholder for future duplicate detection
    DuplicateCards(usize),
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanError::EmptyDocumentSet => write!(f, "No cards found in document set"),
            PlanError::InvalidDeckName(name) => write!(f, "Invalid deck name: {}", name),
            PlanError::DuplicateCards(n) => write!(f, "Duplicate cards found: {} duplicates", n),
        }
    }
}

impl Error for PlanError {}
