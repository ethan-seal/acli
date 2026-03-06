use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::config::{self, ConfigFile};
use crate::error::CliError;
use crate::output::CliOutput;
use crate::sync::{AnkiCli, SyncConfig, ValidationConfig};

#[derive(Debug, Parser)]
#[command(name = "acli")]
#[command(about = "Sync markdown flashcards to Anki")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Sync markdown files in the current directory to an Anki deck.
    Sync {
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

    /// Validate markdown files in the current directory.
    Validate {
        /// Recurse into subdirectories.
        #[arg(short, long)]
        recursive: Option<bool>,
    },

    /// Preview cards that would be synced (alias for sync --dry-run).
    Preview {
        /// Target Anki deck name.
        #[arg(short, long)]
        deck: Option<String>,

        /// Recurse into subdirectories.
        #[arg(short, long)]
        recursive: Option<bool>,
    },

    /// Generate a config file.
    Init {
        /// Create the user-level config (~/.config/acli/config.toml)
        /// instead of a project-level .acli.toml.
        #[arg(long)]
        user: bool,
    },

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

/// Load and merge both config files.  Print a banner line for each file found.
fn load_config_with_banner() -> Result<ConfigFile, CliError> {
    let resolved = config::load_config()?;
    if let Some(ref p) = resolved.user_path {
        eprintln!("Using user config: {}", p.display());
    }
    if let Some(ref p) = resolved.project_path {
        eprintln!("Using project config: {}", p.display());
    }
    Ok(resolved.config)
}

/// Require a value, producing a clear error if missing from both configs and CLI.
fn require<T>(value: Option<T>, field_name: &str) -> Result<T, CliError> {
    value.ok_or_else(|| {
        CliError::ConfigError(format!(
            "`{field_name}` is required. Set it in .acli.toml or pass --{field_name}.",
        ))
    })
}

/// The source directory for every command: the current working directory.
fn source_dir() -> Result<Vec<PathBuf>, CliError> {
    let cwd = std::env::current_dir().map_err(|e| {
        CliError::IoError(std::io::Error::new(
            e.kind(),
            format!("cannot read cwd: {e}"),
        ))
    })?;
    Ok(vec![cwd])
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<(), CliError> {
    let args = Cli::parse();
    let cli = AnkiCli::new();
    let output = CliOutput;

    match args.command {
        Commands::Sync {
            deck,
            collection,
            media_dir,
            recursive,
            dry_run,
        } => {
            let cfg = load_config_with_banner()?;
            let deck_name = require(deck.or(cfg.deck), "deck")?;

            let config = SyncConfig {
                source_dirs: source_dir()?,
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
        Commands::Preview { deck, recursive } => {
            let cfg = load_config_with_banner()?;
            let deck_name = require(deck.or(cfg.deck), "deck")?;

            let config = SyncConfig {
                source_dirs: source_dir()?,
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
        Commands::Validate { recursive } => {
            let cfg = load_config_with_banner()?;

            let config = ValidationConfig {
                source_dirs: source_dir()?,
                recursive: recursive.or(cfg.recursive).unwrap_or(true),
            };

            let result = cli.validate(&config)?;
            output.print_validation_success(&result);
            Ok(())
        }
        Commands::Init { user } => run_init(user),
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

fn run_init(user: bool) -> Result<(), CliError> {
    if user {
        let target = config::user_config_path();
        if target.exists() {
            return Err(CliError::ConfigError(format!(
                "{} already exists",
                target.display()
            )));
        }
        // Create parent directories (e.g. ~/.config/acli/).
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, config::user_starter_config())?;
        println!("Created {}", target.display());
    } else {
        let target = PathBuf::from(config::PROJECT_CONFIG_FILENAME);
        if target.exists() {
            return Err(CliError::ConfigError(format!(
                "{} already exists in the current directory",
                config::PROJECT_CONFIG_FILENAME
            )));
        }
        std::fs::write(&target, config::project_starter_config())?;
        println!("Created {}", target.display());
    }
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
