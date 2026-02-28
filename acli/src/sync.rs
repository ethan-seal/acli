use std::fs;
use std::path::{Path, PathBuf};

use crate::adapter::AnkiCollectionAdapter;
use crate::discovery;
use crate::error::CliError;
use crate::output::SyncResult;
#[cfg(not(feature = "real-anki"))]
use anki_wrapper::FakeAnkiCollection;
use doc_parser::{Card, DocumentParser, MarkdownParser, MediaReference};
use update_planner_parser::{
    DefaultExecutor, DocumentSet, PlanExecutor, SimplePlanner, SyncPlan, UpdatePlanner,
};

#[derive(Debug, Clone, Default)]
pub struct SyncConfig {
    pub source_dirs: Vec<PathBuf>,
    pub deck_name: String,
    pub anki_collection_path: Option<PathBuf>,
    pub anki_media_dir: Option<PathBuf>,
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

        // Collect all media refs for collision checking
        let all_media_refs: Vec<MediaReference> = parsed
            .media_with_dirs
            .iter()
            .flat_map(|(_, refs)| refs.iter().cloned())
            .collect();

        // Check for filename collisions (fail fast)
        crate::media::check_media_collisions(&all_media_refs)
            .map_err(CliError::MediaCollisionError)?;

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

        // Copy media files if anki_media_dir is configured
        let mut total_media_copied = 0;
        let mut total_media_skipped = 0;
        let mut total_media_missing = 0;

        if let Some(media_dir) = &config.anki_media_dir {
            for (doc_dir, media_refs) in &parsed.media_with_dirs {
                if !media_refs.is_empty() {
                    let report = crate::media::copy_media_to_anki(doc_dir, media_refs, media_dir)?;
                    total_media_copied += report.copied;
                    total_media_skipped += report.skipped;
                    total_media_missing += report.missing.len();
                }
            }
        }

        // Execute plan against Anki collection
        self.execute_plan(&plan, config)?;

        let mut result = SyncResult::new(
            markdown_files.len(),
            plan.operations.len(),
            config.deck_name.clone(),
            false,
        );
        result.media_copied = total_media_copied;
        result.media_skipped = total_media_skipped;
        result.media_missing = total_media_missing;
        Ok(result)
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
        let mut media_with_dirs = Vec::new();

        for path in files {
            let content = fs::read_to_string(path)?;
            let parsed = self.parser.parse(&content)?;
            cards.extend(parsed.cards);
            source_files.push(path.display().to_string());

            // Collect media refs with the document's parent directory
            let doc_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            media_with_dirs.push((doc_dir, parsed.media));
        }

        Ok(ParsedBatch {
            cards,
            source_files,
            media_with_dirs,
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
    /// Per-document (document_dir, media_refs) pairs.
    media_with_dirs: Vec<(PathBuf, Vec<MediaReference>)>,
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
