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

    #[error("feature not yet implemented")]
    NotImplemented,
}
