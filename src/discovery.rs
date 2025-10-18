use std::path::PathBuf;

use crate::error::CliError;

pub fn discover_markdown_files(_sources: &[PathBuf]) -> Result<Vec<PathBuf>, CliError> {
    // Placeholder implementation; real discovery logic implemented later.
    todo!("File discovery not yet implemented")
}
