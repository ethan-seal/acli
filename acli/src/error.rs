use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Directory traversal error: {0}")]
    WalkdirError(#[from] walkdir::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] doc_parser::ParseError),

    #[error("Plan error: {0}")]
    PlanError(#[from] update_planner_parser::PlanError),

    #[error("Anki error: {0}")]
    AnkiError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("feature not yet implemented")]
    NotImplemented,
}
