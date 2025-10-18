use std::path::PathBuf;

use crate::error::CliError;
use crate::output::SyncResult;

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub source_dirs: Vec<PathBuf>,
    pub deck_name: String,
    pub anki_collection_path: Option<PathBuf>,
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

    pub fn validate(&self, _config: &SyncConfig) -> Result<(), CliError> {
        Err(CliError::NotImplemented)
    }
}
