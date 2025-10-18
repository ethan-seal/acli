use std::path::{Path, PathBuf};

use crate::discovery;
use crate::error::CliError;
use crate::output::SyncResult;

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub source_dirs: Vec<PathBuf>,
    pub deck_name: String,
    pub anki_collection_path: Option<PathBuf>,
    pub recursive: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub source_dirs: Vec<PathBuf>,
    pub recursive: bool,
}

#[derive(Default)]
pub struct AnkiCli;

impl AnkiCli {
    pub fn new() -> Self {
        Self
    }

    pub fn sync(&self, _config: &SyncConfig) -> Result<SyncResult, CliError> {
        Err(CliError::NotImplemented)
    }

    pub fn preview(&self, _config: &SyncConfig) -> Result<SyncResult, CliError> {
        Err(CliError::NotImplemented)
    }

    pub fn validate(&self, _config: &ValidationConfig) -> Result<(), CliError> {
        Err(CliError::NotImplemented)
    }

    pub fn discover_files(
        &self,
        sources: &[PathBuf],
        recursive: bool,
    ) -> Result<Vec<PathBuf>, CliError> {
        discovery::discover_markdown_files(sources, recursive)
    }

    pub fn is_markdown_file(&self, path: &Path) -> bool {
        discovery::is_markdown_file(path)
    }
}
