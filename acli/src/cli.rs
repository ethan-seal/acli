use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

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
    /// Sync markdown files to Anki deck
    Sync {
        /// Directory containing markdown files
        #[arg(short, long)]
        source: PathBuf,

        /// Target Anki deck name
        #[arg(short, long)]
        deck: String,

        /// Anki collection path (optional, uses default if not specified)
        #[arg(short, long)]
        collection: Option<PathBuf>,

        /// Recursive directory traversal
        #[arg(short, long, action = ArgAction::Set, default_value_t = true)]
        recursive: bool,
    },

    /// Validate markdown files without syncing
    Validate {
        /// Directory containing markdown files
        #[arg(short, long)]
        source: PathBuf,

        /// Recursive directory traversal
        #[arg(short, long, action = ArgAction::Set, default_value_t = true)]
        recursive: bool,
    },

    /// List all cards that would be synced
    Preview {
        /// Directory containing markdown files
        #[arg(short, long)]
        source: PathBuf,

        /// Target Anki deck name
        #[arg(short, long)]
        deck: String,

        /// Recursive directory traversal
        #[arg(short, long, action = ArgAction::Set, default_value_t = true)]
        recursive: bool,
    },

    /// Record a review for a card by its Anki database ID
    ReviewCard {
        /// Anki database card ID
        #[arg(long)]
        card_id: i64,

        /// Review rating
        #[arg(long)]
        rating: RatingArg,

        /// Anki collection path
        #[arg(short, long)]
        collection: PathBuf,
    },

    /// Get review history for a card by its Anki database ID (outputs JSON)
    GetReviews {
        /// Anki database card ID
        #[arg(long)]
        card_id: i64,

        /// Anki collection path
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

pub fn run() -> Result<(), CliError> {
    let args = Cli::parse();
    let cli = AnkiCli::new();
    let output = CliOutput;

    match args.command {
        Commands::Sync {
            source,
            deck,
            collection,
            recursive,
        } => {
            let config = SyncConfig {
                source_dirs: vec![source],
                deck_name: deck,
                anki_collection_path: collection,
                anki_media_dir: None,
                recursive,
                dry_run: false,
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
            let config = SyncConfig {
                source_dirs: vec![source],
                deck_name: deck,
                anki_collection_path: None,
                anki_media_dir: None,
                recursive,
                dry_run: true,
            };

            let result = cli.preview(&config)?;
            output.print_sync_result(&result);
            Ok(())
        }
        Commands::Validate { source, recursive } => {
            let config = ValidationConfig {
                source_dirs: vec![source],
                recursive,
            };

            let files_checked = cli.validate(&config)?;
            output.print_validation_success(files_checked);
            Ok(())
        }
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
