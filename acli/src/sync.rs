use std::fs;
use std::path::{Path, PathBuf};

use crate::adapter::{convert_card as convert_to_anki_card, AnkiCollectionAdapter};
use crate::discovery;
use crate::error::CliError;
use crate::output::SyncResult;
use anki_wrapper::{AnkiCollection as AnkiWrapperCollection, CardInfo, CardType as AnkiCardType};
use doc_parser::{Card, DocumentParser, MarkdownParser, MediaReference};

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

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub total_files: usize,
    pub files_with_cards: usize,
}

/// Result of previewing cards that would be synced.
#[derive(Debug, Clone)]
pub struct PreviewResult {
    pub files_processed: usize,
    pub deck_name: String,
    pub cards: Vec<Card>,
}

/// A content-based card identity used for matching parsed cards against Anki cards.
/// Two cards are "the same" if they have the same type and fields.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CardContentKey {
    card_type: AnkiCardType,
    fields: Vec<String>,
}

/// Result of diffing parsed cards against what's currently in Anki.
struct IncrementalDiff {
    /// Parsed cards to add (not currently in Anki).
    to_add: Vec<Card>,
    /// Anki database IDs of cards to delete (in Anki but not in parsed).
    to_delete: Vec<anki_wrapper::CardId>,
    /// Number of cards that are unchanged (in both parsed and Anki).
    unchanged: usize,
}

/// Compute the diff between parsed cards and the current Anki deck state.
///
/// Cards are matched by content: (card_type, fields) after HTML conversion.
/// - Parsed cards not found in Anki → to_add
/// - Anki cards not found in parsed → to_delete
/// - Cards in both → unchanged (reviews preserved)
fn compute_incremental_diff(parsed_cards: &[Card], anki_cards: &[CardInfo]) -> IncrementalDiff {
    // Convert parsed cards to their Anki representation (HTML fields + mapped type)
    // so we compare apples to apples.
    let parsed_keys: Vec<CardContentKey> = parsed_cards
        .iter()
        .map(|c| {
            let anki_card = convert_to_anki_card(&to_planner_card(c));
            CardContentKey {
                card_type: anki_card.card_type,
                fields: anki_card.fields,
            }
        })
        .collect();

    // Build content keys for what's currently in Anki
    let anki_keys: Vec<CardContentKey> = anki_cards
        .iter()
        .map(|c| CardContentKey {
            card_type: c.card_type.clone(),
            fields: c.fields.clone(),
        })
        .collect();

    // Use multiset-style matching to handle duplicate cards correctly.
    // Track which Anki cards have been matched (by index).
    let mut anki_matched: Vec<bool> = vec![false; anki_cards.len()];
    let mut parsed_matched: Vec<bool> = vec![false; parsed_cards.len()];

    // Match parsed cards to Anki cards
    for (pi, pk) in parsed_keys.iter().enumerate() {
        for (ai, ak) in anki_keys.iter().enumerate() {
            if !anki_matched[ai] && pk == ak {
                anki_matched[ai] = true;
                parsed_matched[pi] = true;
                break;
            }
        }
    }

    let to_add: Vec<Card> = parsed_cards
        .iter()
        .enumerate()
        .filter(|(i, _)| !parsed_matched[*i])
        .map(|(_, c)| c.clone())
        .collect();

    let to_delete: Vec<anki_wrapper::CardId> = anki_cards
        .iter()
        .enumerate()
        .filter(|(i, _)| !anki_matched[*i])
        .map(|(_, c)| c.id)
        .collect();

    let unchanged = parsed_matched.iter().filter(|m| **m).count();

    IncrementalDiff {
        to_add,
        to_delete,
        unchanged,
    }
}

/// Convert a doc_parser::Card to an update_planner_parser::Card.
fn to_planner_card(card: &Card) -> update_planner_parser::Card {
    let card_type = match card.card_type {
        doc_parser::CardType::Basic => update_planner_parser::CardType::Basic,
        doc_parser::CardType::Bidirectional => update_planner_parser::CardType::Bidirectional,
        doc_parser::CardType::Sequence => update_planner_parser::CardType::Sequence,
    };
    update_planner_parser::Card {
        card_type,
        fields: card.fields.clone(),
    }
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

        if config.dry_run {
            return Ok(SyncResult::new(
                markdown_files.len(),
                parsed.cards.len(),
                config.deck_name.clone(),
                true,
            ));
        }

        // Refuse to sync when the real Anki backend is not compiled in.
        // Without it, cards would be "synced" to an in-memory fake and then
        // silently discarded — misleading the user into thinking they saved.
        #[cfg(not(feature = "real-anki"))]
        return Err(CliError::AnkiError(
            "sync requires building with --features real-anki. \
             Use `acli sync --dry-run` or `acli preview` to verify your cards."
                .to_string(),
        ));

        #[cfg(feature = "real-anki")]
        {
            // Copy media files if anki_media_dir is configured
            let mut total_media_copied = 0;
            let mut total_media_skipped = 0;
            let mut total_media_missing = 0;

            if let Some(media_dir) = &config.anki_media_dir {
                for (doc_dir, media_refs) in &parsed.media_with_dirs {
                    if !media_refs.is_empty() {
                        let report =
                            crate::media::copy_media_to_anki(doc_dir, media_refs, media_dir)?;
                        total_media_copied += report.copied;
                        total_media_skipped += report.skipped;
                        total_media_missing += report.missing.len();
                    }
                }
            }

            // Execute incremental sync against Anki collection
            let sync_counts = {
                use anki_wrapper::DefaultAnkiCollection;
                let collection = if let Some(path) = &config.anki_collection_path {
                    DefaultAnkiCollection::open_collection_path(path)
                        .map_err(|e| CliError::AnkiError(e.to_string()))?
                } else {
                    DefaultAnkiCollection::new().map_err(|e| CliError::AnkiError(e.to_string()))?
                };
                let mut adapter = AnkiCollectionAdapter::new(collection);
                sync_incremental(&parsed.cards, &config.deck_name, &mut adapter)
                    .map_err(|e| CliError::ExecutionError(e.to_string()))?
            };

            let mut result = SyncResult::new(
                markdown_files.len(),
                sync_counts.added + sync_counts.deleted,
                config.deck_name.clone(),
                false,
            );
            result.cards_added = sync_counts.added;
            result.cards_deleted = sync_counts.deleted;
            result.cards_unchanged = sync_counts.unchanged;
            result.media_copied = total_media_copied;
            result.media_skipped = total_media_skipped;
            result.media_missing = total_media_missing;
            Ok(result)
        }
    }

    /// Parse all markdown files and return the cards that would be synced.
    pub fn preview(&self, config: &SyncConfig) -> Result<PreviewResult, CliError> {
        let markdown_files = self.discover_files(&config.source_dirs, config.recursive)?;
        let parsed = self.parse_documents(&markdown_files)?;

        // Check for media collisions (same as sync — surface errors early).
        let all_media_refs: Vec<MediaReference> = parsed
            .media_with_dirs
            .iter()
            .flat_map(|(_, refs)| refs.iter().cloned())
            .collect();
        crate::media::check_media_collisions(&all_media_refs)
            .map_err(CliError::MediaCollisionError)?;

        Ok(PreviewResult {
            files_processed: markdown_files.len(),
            deck_name: config.deck_name.clone(),
            cards: parsed.cards,
        })
    }

    pub fn validate(&self, config: &ValidationConfig) -> Result<ValidationResult, CliError> {
        let markdown_files = self.discover_files(&config.source_dirs, config.recursive)?;
        let parsed = self.parse_documents(&markdown_files)?;
        Ok(ValidationResult {
            total_files: markdown_files.len(),
            files_with_cards: parsed.source_files.len(),
        })
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
            match self.parser.parse(&content) {
                Ok(parsed) => {
                    // Surface non-fatal warnings (e.g. incomplete block cards).
                    for warning in &parsed.warnings {
                        eprintln!("warning: {}: {}", path.display(), warning);
                    }

                    cards.extend(parsed.cards);
                    source_files.push(path.display().to_string());

                    // Collect media refs with the document's parent directory
                    let doc_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
                    media_with_dirs.push((doc_dir, parsed.media));
                }
                // Files with no cards are silently skipped
                Err(doc_parser::ParseError::EmptyDocument) => continue,
                Err(e) => {
                    return Err(CliError::ParseError(format!("{}: {}", path.display(), e)));
                }
            }
        }

        Ok(ParsedBatch {
            cards,
            source_files,
            media_with_dirs,
        })
    }
}

/// Counts of what happened during an incremental sync.
pub struct SyncCounts {
    pub added: usize,
    pub deleted: usize,
    pub unchanged: usize,
}

/// Perform an incremental sync: read current Anki state, diff against parsed cards,
/// and only add/delete what changed. Unchanged cards (and their reviews) are preserved.
pub fn sync_incremental<C: AnkiWrapperCollection>(
    parsed_cards: &[Card],
    deck_name: &str,
    adapter: &mut AnkiCollectionAdapter<C>,
) -> Result<SyncCounts, Box<dyn std::error::Error>> {
    // 1. Ensure deck exists
    adapter
        .ensure_deck(deck_name)
        .map_err(|e| format!("Failed to ensure deck: {}", e))?;

    // 2. Read current cards from Anki
    let anki_cards = adapter
        .get_cards_in_deck(deck_name)
        .map_err(|e| format!("Failed to read deck: {}", e))?;

    // 3. Compute diff
    let diff = compute_incremental_diff(parsed_cards, &anki_cards);

    // 4. Delete cards that are no longer in the markdown
    for card_id in &diff.to_delete {
        adapter
            .delete_card_by_anki_id(*card_id)
            .map_err(|e| format!("Failed to delete card: {}", e))?;
    }

    // 5. Add new cards from the markdown
    for card in &diff.to_add {
        let planner_card = to_planner_card(card);
        adapter
            .add_card_to_deck(deck_name, &planner_card)
            .map_err(|e| format!("Failed to add card: {}", e))?;
    }

    Ok(SyncCounts {
        added: diff.to_add.len(),
        deleted: diff.to_delete.len(),
        unchanged: diff.unchanged,
    })
}

struct ParsedBatch {
    cards: Vec<Card>,
    #[allow(dead_code)]
    source_files: Vec<String>,
    /// Per-document (document_dir, media_refs) pairs.
    media_with_dirs: Vec<(PathBuf, Vec<MediaReference>)>,
}
