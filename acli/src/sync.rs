use std::fs;
use std::path::{Path, PathBuf};

use crate::discovery;
use doc_parser::{Card, DocumentParser, MarkdownParser};
use crate::error::CliError;
use crate::output::SyncResult;

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub source_dirs: Vec<PathBuf>,
    pub deck_name: String,
    pub anki_collection_path: Option<PathBuf>,
    pub recursive: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub source_dirs: Vec<PathBuf>,
    pub recursive: bool,
}

#[derive(Default)]
pub struct AnkiCli {
    parser: MarkdownParser,
}

impl AnkiCli {
    pub fn new() -> Self {
        Self {
            parser: MarkdownParser::new(),
        }
    }

    pub fn sync(&self, config: &SyncConfig) -> Result<SyncResult, CliError> {
        let markdown_files = self.discover_files(&config.source_dirs, config.recursive)?;
        let parsed = self.parse_documents(&markdown_files)?;

        if config.dry_run {
            return Ok(SyncResult::new(
                markdown_files.len(),
                parsed.cards.len(),
                config.deck_name.clone(),
                true,
            ));
        }

        // TODO: wire into update planner and Anki collection once dependencies are available.

        Ok(SyncResult::new(
            markdown_files.len(),
            parsed.cards.len(),
            config.deck_name.clone(),
            false,
        ))
    }

    pub fn preview(&self, config: &SyncConfig) -> Result<SyncResult, CliError> {
        self.sync(config)
    }

    pub fn validate(&self, config: &ValidationConfig) -> Result<usize, CliError> {
        let markdown_files = self.discover_files(&config.source_dirs, config.recursive)?;
        let _ = self.parse_documents(&markdown_files)?;
        Ok(markdown_files.len())
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

    fn parse_documents(&self, files: &[PathBuf]) -> Result<ParsedBatch, CliError> {
        let mut cards = Vec::new();

        for path in files {
            let content = fs::read_to_string(path)?;
            let parsed = self.parser.parse(&content)?;
            cards.extend(parsed.cards);
        }

        Ok(ParsedBatch { cards })
    }
}

struct ParsedBatch {
    cards: Vec<Card>,
}
