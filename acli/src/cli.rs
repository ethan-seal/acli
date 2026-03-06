use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{self, ConfigFile};
use crate::error::CliError;
use crate::output::CliOutput;
use crate::sync::{AnkiCli, SyncConfig, ValidationConfig};

#[derive(Debug, Parser)]
#[command(name = "acli")]
#[command(about = "Sync markdown documents to Anki")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Sync markdown files to Anki deck.
    /// Reads defaults from .acli.toml if present.
    Sync {
        /// Source directories containing markdown files (repeatable).
        /// Overrides config file `source` when provided.
        #[arg(short, long)]
        source: Vec<PathBuf>,

        /// Target Anki deck name.
        #[arg(short, long)]
        deck: Option<String>,

        /// Anki collection path (uses default if not specified).
        #[arg(short, long)]
        collection: Option<PathBuf>,

        /// Path to Anki's collection.media directory for image syncing.
        #[arg(short, long)]
        media_dir: Option<PathBuf>,

        /// Recurse into subdirectories.
        #[arg(short, long)]
        recursive: Option<bool>,

        /// Show what would be synced without making changes.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate markdown files without syncing.
    Validate {
        /// Source directories containing markdown files (repeatable).
        #[arg(short, long)]
        source: Vec<PathBuf>,

        /// Recurse into subdirectories.
        #[arg(short, long)]
        recursive: Option<bool>,
    },

    /// Preview cards that would be synced (alias for sync --dry-run).
    Preview {
        /// Source directories containing markdown files (repeatable).
        #[arg(short, long)]
        source: Vec<PathBuf>,

        /// Target Anki deck name.
        #[arg(short, long)]
        deck: Option<String>,

        /// Recurse into subdirectories.
        #[arg(short, long)]
        recursive: Option<bool>,
    },

    /// Generate a starter .acli.toml in the current directory.
    Init,

    /// Record a review for a card by its Anki database ID.
    ReviewCard {
        /// Anki database card ID.
        #[arg(long)]
        card_id: i64,

        /// Review rating.
        #[arg(long)]
        rating: RatingArg,

        /// Anki collection path.
        #[arg(short, long)]
        collection: PathBuf,
    },

    /// Get review history for a card by its Anki database ID (outputs JSON).
    GetReviews {
        /// Anki database card ID.
        #[arg(long)]
        card_id: i64,

        /// Anki collection path.
        #[arg(short, long)]
        collection: PathBuf,
    },
}

/// CLI-friendly review rating values.
#[derive(Debug, Clone, ValueEnum)]
enum RatingArg {
    Again,
    Hard,
    Good,
    Easy,
}

impl From<RatingArg> for anki_wrapper::ReviewRating {
    fn from(r: RatingArg) -> Self {
        match r {
            RatingArg::Again => anki_wrapper::ReviewRating::Again,
            RatingArg::Hard => anki_wrapper::ReviewRating::Hard,
            RatingArg::Good => anki_wrapper::ReviewRating::Good,
            RatingArg::Easy => anki_wrapper::ReviewRating::Easy,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Load the config file (if any) and print which one we found.
fn load_config_with_banner() -> Result<(ConfigFile, Option<PathBuf>), CliError> {
    match config::load_config()? {
        Some((cfg, path)) => {
            eprintln!("Using config: {}", path.display());
            let config_dir = path.parent().unwrap_or(std::path::Path::new("."));
            // Resolve relative source paths against the config file's directory.
            let resolved = ConfigFile {
                source: config::resolve_source_paths(config_dir, &cfg.source),
                ..cfg
            };
            Ok((resolved, Some(path)))
        }
        None => Ok((ConfigFile::default(), None)),
    }
}

/// Require a value, producing a clear error if missing from both CLI and config.
fn require<T>(value: Option<T>, field_name: &str) -> Result<T, CliError> {
    value.ok_or_else(|| {
        CliError::ConfigError(format!(
            "`{field_name}` is required. Set it in .acli.toml or pass --{field_name}.",
        ))
    })
}

/// Merge CLI source dirs with config.  CLI wins if non-empty.
fn merge_sources(cli: Vec<PathBuf>, config: Vec<PathBuf>) -> Vec<PathBuf> {
    if cli.is_empty() {
        config
    } else {
        cli
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<(), CliError> {
    let args = Cli::parse();
    let cli = AnkiCli::new();
    let output = CliOutput;

    match args.command {
        Commands::Sync {
            source,
            deck,
            collection,
            media_dir,
            recursive,
            dry_run,
        } => {
            let (cfg, _) = load_config_with_banner()?;

            let source_dirs = merge_sources(source, cfg.source);
            if source_dirs.is_empty() {
                return Err(CliError::ConfigError(
                    "`source` is required. Set it in .acli.toml or pass --source.".into(),
                ));
            }
            let deck_name = require(deck.or(cfg.deck), "deck")?;

            let config = SyncConfig {
                source_dirs,
                deck_name,
                anki_collection_path: collection.or(cfg.collection),
                anki_media_dir: media_dir.or(cfg.media_dir),
                recursive: recursive.or(cfg.recursive).unwrap_or(true),
                dry_run,
            };

            let result = cli.sync(&config)?;
            output.print_sync_result(&result);
            Ok(())
        }
        Commands::Preview {
            source,
            deck,
            recursive,
        } => {
            let (cfg, _) = load_config_with_banner()?;

            let source_dirs = merge_sources(source, cfg.source);
            if source_dirs.is_empty() {
                return Err(CliError::ConfigError(
                    "`source` is required. Set it in .acli.toml or pass --source.".into(),
                ));
            }
            let deck_name = require(deck.or(cfg.deck), "deck")?;

            let config = SyncConfig {
                source_dirs,
                deck_name,
                anki_collection_path: None,
                anki_media_dir: None,
                recursive: recursive.or(cfg.recursive).unwrap_or(true),
                dry_run: true,
            };

            let result = cli.preview(&config)?;
            output.print_sync_result(&result);
            Ok(())
        }
        Commands::Validate { source, recursive } => {
            let (cfg, _) = load_config_with_banner()?;

            let source_dirs = merge_sources(source, cfg.source);
            if source_dirs.is_empty() {
                return Err(CliError::ConfigError(
                    "`source` is required. Set it in .acli.toml or pass --source.".into(),
                ));
            }

            let config = ValidationConfig {
                source_dirs,
                recursive: recursive.or(cfg.recursive).unwrap_or(true),
            };

            let result = cli.validate(&config)?;
            output.print_validation_success(&result);
            Ok(())
        }
        Commands::Init => run_init(),
        Commands::ReviewCard {
            card_id,
            rating,
            collection,
        } => run_review_card(card_id, rating, &collection),
        Commands::GetReviews {
            card_id,
            collection,
        } => run_get_reviews(card_id, &collection),
    }
}

// ── Subcommand implementations ────────────────────────────────────────────────

fn run_init() -> Result<(), CliError> {
    let target = PathBuf::from(config::CONFIG_FILENAME);
    if target.exists() {
        return Err(CliError::ConfigError(format!(
            "{} already exists in the current directory",
            config::CONFIG_FILENAME
        )));
    }
    std::fs::write(&target, config::starter_config())?;
    println!("Created {}", target.display());
    Ok(())
}

fn run_review_card(
    card_id: i64,
    rating: RatingArg,
    collection_path: &PathBuf,
) -> Result<(), CliError> {
    let anki_card_id = anki_wrapper::CardId(card_id);
    let anki_rating: anki_wrapper::ReviewRating = rating.into();

    #[cfg(feature = "real-anki")]
    {
        use anki_wrapper::{AnkiCollection, DefaultAnkiCollection};
        let mut collection = DefaultAnkiCollection::open_collection_path(collection_path)
            .map_err(|e| CliError::AnkiError(e.to_string()))?;
        collection
            .record_review(anki_card_id, anki_rating)
            .map_err(|e| CliError::AnkiError(e.to_string()))?;
        println!(
            "Recorded review for card {} with rating {:?}",
            card_id, anki_rating
        );
        Ok(())
    }

    #[cfg(not(feature = "real-anki"))]
    {
        let _ = (anki_card_id, anki_rating, collection_path);
        Err(CliError::AnkiError(
            "review-card requires the real-anki feature".to_string(),
        ))
    }
}

fn run_get_reviews(card_id: i64, collection_path: &PathBuf) -> Result<(), CliError> {
    let anki_card_id = anki_wrapper::CardId(card_id);

    #[cfg(feature = "real-anki")]
    {
        use anki_wrapper::{AnkiCollection, DefaultAnkiCollection};
        let mut collection = DefaultAnkiCollection::open_collection_path(collection_path)
            .map_err(|e| CliError::AnkiError(e.to_string()))?;
        let reviews = collection
            .get_reviews(anki_card_id)
            .map_err(|e| CliError::AnkiError(e.to_string()))?;

        // Output as JSON for easy parsing by scripts
        let json_entries: Vec<serde_json::Value> = reviews
            .iter()
            .map(|r| {
                serde_json::json!({
                    "card_id": r.card_id.0,
                    "rating": format!("{:?}", r.rating),
                    "interval": r.interval,
                    "ease_factor": r.ease_factor,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json_entries)
                .map_err(|e| CliError::AnkiError(e.to_string()))?
        );
        Ok(())
    }

    #[cfg(not(feature = "real-anki"))]
    {
        let _ = (anki_card_id, collection_path);
        Err(CliError::AnkiError(
            "get-reviews requires the real-anki feature".to_string(),
        ))
    }
}
