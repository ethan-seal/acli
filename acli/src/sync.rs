use std::fs;
use std::path::{Path, PathBuf};

use crate::adapter::AnkiCollectionAdapter;
use crate::discovery;
use crate::error::CliError;
use crate::output::SyncResult;
#[cfg(not(feature = "real-anki"))]
use anki_wrapper::FakeAnkiCollection;
use doc_parser::{Card, DocumentParser, MarkdownParser};
use update_planner_parser::{
    DefaultExecutor, DocumentSet, PlanExecutor, SimplePlanner, SyncPlan, UpdatePlanner,
};

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

        // Convert to DocumentSet and generate sync plan
        let document_set = to_document_set(&parsed);
        let plan = self.plan_sync(&document_set, &config.deck_name)?;

        if config.dry_run {
            return Ok(SyncResult::new(
                markdown_files.len(),
                plan.operations.len(),
                config.deck_name.clone(),
                true,
            ));
        }

        // Execute plan against Anki collection
        self.execute_plan(&plan, config)?;

        Ok(SyncResult::new(
            markdown_files.len(),
            plan.operations.len(),
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
        let mut source_files = Vec::new();

        for path in files {
            let content = fs::read_to_string(path)?;
            let parsed = self.parser.parse(&content)?;
            cards.extend(parsed.cards);
            source_files.push(path.display().to_string());
        }

        Ok(ParsedBatch {
            cards,
            source_files,
        })
    }

    fn plan_sync(&self, document_set: &DocumentSet, deck_name: &str) -> Result<SyncPlan, CliError> {
        let planner = SimplePlanner;
        let plan = planner.plan_fresh_sync(document_set, deck_name)?;
        Ok(plan)
    }

    fn execute_plan(&self, plan: &SyncPlan, config: &SyncConfig) -> Result<(), CliError> {
        let executor = DefaultExecutor;

        // Create the appropriate collection based on config
        // For now, we use FakeAnkiCollection since real-anki feature requires
        // building outside the workspace. When real-anki is enabled, this would
        // use DefaultAnkiCollection::open_collection_path() instead.
        #[cfg(feature = "real-anki")]
        {
            use anki_wrapper::DefaultAnkiCollection;
            let collection = if let Some(path) = &config.anki_collection_path {
                DefaultAnkiCollection::open_collection_path(path)
                    .map_err(|e| CliError::AnkiError(e.to_string()))?
            } else {
                DefaultAnkiCollection::new().map_err(|e| CliError::AnkiError(e.to_string()))?
            };
            let mut adapter = AnkiCollectionAdapter::new(collection);
            executor
                .execute_plan(plan, &mut adapter)
                .map_err(|e| CliError::ExecutionError(e.to_string()))?;
        }

        #[cfg(not(feature = "real-anki"))]
        {
            // Without real-anki feature, we can only use FakeAnkiCollection
            // This is useful for testing the pipeline without a real Anki installation
            let _ = config; // suppress unused warning
            let mut adapter = AnkiCollectionAdapter::new(FakeAnkiCollection::new());
            executor
                .execute_plan(plan, &mut adapter)
                .map_err(|e| CliError::ExecutionError(e.to_string()))?;
        }

        Ok(())
    }
}

struct ParsedBatch {
    cards: Vec<Card>,
    source_files: Vec<String>,
}

/// Convert a doc_parser::Card to an update_planner_parser::Card.
fn convert_card(card: &Card) -> update_planner_parser::Card {
    let card_type = match card.card_type {
        doc_parser::CardType::Basic => update_planner_parser::CardType::Basic,
        doc_parser::CardType::Bidirectional => update_planner_parser::CardType::Bidirectional,
    };
    update_planner_parser::Card {
        card_type,
        fields: card.fields.clone(),
    }
}

/// Convert parsed cards to a DocumentSet for the planner.
fn to_document_set(batch: &ParsedBatch) -> DocumentSet {
    DocumentSet {
        cards: batch.cards.iter().map(convert_card).collect(),
        source_files: batch.source_files.clone(),
    }
}
