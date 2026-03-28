use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use anyhow::{Context, Result, anyhow, bail};

use crate::config::{self, ConfigFile};
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

        /// Do not recurse into subdirectories.
        #[arg(long)]
        no_recursive: bool,

        /// Show what would be synced without making changes.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate markdown files in the current directory.
    Validate {
        /// Do not recurse into subdirectories.
        #[arg(long)]
        no_recursive: bool,
    },

    /// Preview cards that would be synced (alias for sync --dry-run).
    Preview {
        /// Target Anki deck name.
        #[arg(short, long)]
        deck: Option<String>,

        /// Do not recurse into subdirectories.
        #[arg(long)]
        no_recursive: bool,
    },

    /// Start a local web server to preview cards in a browser.
    ///
    /// Re-parses markdown files on every page load, so edits are reflected
    /// immediately on browser refresh.  Requires --features web.
    Serve {
        /// Target Anki deck name (used for display only).
        #[arg(short, long)]
        deck: Option<String>,

        /// Port to listen on.
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Do not recurse into subdirectories.
        #[arg(long)]
        no_recursive: bool,
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
fn load_config_with_banner() -> Result<ConfigFile> {
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
fn require<T>(value: Option<T>, field_name: &str) -> Result<T> {
    value.ok_or_else(|| {
        anyhow!("`{field_name}` is required. Set it in .acli.toml or pass --{field_name}.")
    })
}

/// Validate a deck name, returning a clear error for common mistakes.
fn validate_deck_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("deck name cannot be empty");
    }
    if trimmed.contains('/') {
        bail!(
            "deck name '{}' contains '/'. \
             Use '::' for hierarchical decks (e.g. 'Languages::Spanish').",
            name
        );
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        bail!(
            "deck name '{}' contains invalid whitespace characters",
            name
        );
    }
    Ok(())
}

/// The source directory for every command: the current working directory.
fn source_dir() -> Result<Vec<PathBuf>> {
    let cwd = std::env::current_dir().context("cannot read cwd")?;
    Ok(vec![cwd])
}

/// Resolved configuration values for commands requiring deck and recursive settings.
struct ResolvedDeckConfig {
    deck_name: String,
    recursive: bool,
    collection: Option<PathBuf>,
    media_dir: Option<PathBuf>,
}

/// Resolve configuration for Sync and Preview commands.
///
/// Loads config, resolves and validates deck name, resolves recursive flag,
/// and merges CLI args with config file values.
fn resolve_deck_config(
    deck: Option<String>,
    collection: Option<PathBuf>,
    media_dir: Option<PathBuf>,
    no_recursive: bool,
) -> Result<ResolvedDeckConfig> {
    let cfg = load_config_with_banner()?;
    let deck_name = require(deck.or(cfg.deck), "deck")?;
    validate_deck_name(&deck_name)?;
    let recursive = if no_recursive {
        false
    } else {
        cfg.recursive.unwrap_or(true)
    };

    Ok(ResolvedDeckConfig {
        deck_name,
        recursive,
        collection: collection.or(cfg.collection),
        media_dir: media_dir.or(cfg.media_dir),
    })
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    let args = Cli::parse();
    let cli = AnkiCli::new();
    let output = CliOutput;

    match args.command {
        Commands::Sync {
            deck,
            collection,
            media_dir,
            no_recursive,
            dry_run,
        } => {
            let resolved = resolve_deck_config(deck, collection, media_dir, no_recursive)?;

            let config = SyncConfig {
                source_dirs: source_dir()?,
                deck_name: resolved.deck_name,
                anki_collection_path: resolved.collection,
                anki_media_dir: resolved.media_dir,
                recursive: resolved.recursive,
                dry_run,
            };

            let result = cli.sync(&config)?;
            output.print_sync_result(&result);
            Ok(())
        }
        Commands::Preview { deck, no_recursive } => {
            let resolved = resolve_deck_config(deck, None, None, no_recursive)?;

            let config = SyncConfig {
                source_dirs: source_dir()?,
                deck_name: resolved.deck_name,
                anki_collection_path: None,
                anki_media_dir: None,
                recursive: resolved.recursive,
                dry_run: true,
            };

            let result = cli.preview(&config)?;
            output.print_preview(&result);
            Ok(())
        }
        Commands::Validate { no_recursive } => {
            let cfg = load_config_with_banner()?;
            let recursive = if no_recursive {
                false
            } else {
                cfg.recursive.unwrap_or(true)
            };

            let config = ValidationConfig {
                source_dirs: source_dir()?,
                recursive,
            };

            let result = cli.validate(&config)?;
            output.print_validation_success(&result);
            Ok(())
        }
        Commands::Serve {
            deck,
            port,
            no_recursive,
        } => run_serve(deck, port, no_recursive),
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

// ── Serve ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "web")]
fn discover_card_files(
    source_dirs: &[std::path::PathBuf],
    recursive: bool,
) -> Result<Vec<std::path::PathBuf>, String> {
    AnkiCli::new()
        .discover_files(source_dirs, recursive)
        .map_err(|e| e.to_string())
}

#[cfg(feature = "web")]
fn parse_and_convert_cards(
    files: &[std::path::PathBuf],
    cwd: &Option<std::path::PathBuf>,
) -> (Vec<web_preview::PreviewCard>, Vec<String>) {
    use doc_parser::{DocumentParser, MarkdownParser};

    let parser = MarkdownParser::new();
    let mut cards = Vec::new();
    let mut errors = Vec::new();

    for path in files {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("{}: {e}", path.display()));
                continue;
            }
        };
        match parser.parse(&content) {
            Ok(parsed) => {
                for w in &parsed.warnings {
                    errors.push(format!("{}: {w}", path.display()));
                }
                for card in parsed.cards {
                    let display_path = match cwd {
                        Some(ref cwd) => {
                            path.strip_prefix(cwd).unwrap_or(path).display().to_string()
                        }
                        None => path.display().to_string(),
                    };
                    let source_dir = path.parent().unwrap_or(std::path::Path::new("."));
                    let source_dir_rel = match cwd {
                        Some(ref cwd) => source_dir.strip_prefix(cwd).unwrap_or(source_dir),
                        None => source_dir,
                    };
                    let front_raw = card.fields.first().map(|s| s.as_str()).unwrap_or("");
                    let back_raw = card.fields.get(1).map(|s| s.as_str()).unwrap_or("");
                    cards.push(web_preview::PreviewCard {
                        front: web_preview::rewrite_image_urls(front_raw, source_dir_rel),
                        back: web_preview::rewrite_image_urls(back_raw, source_dir_rel),
                        card_type: match card.card_type {
                            doc_parser::CardType::Basic => web_preview::CardType::Basic,
                            doc_parser::CardType::Bidirectional => {
                                web_preview::CardType::Bidirectional
                            }
                            doc_parser::CardType::Sequence => web_preview::CardType::Sequence,
                        },
                        source_file: Some(display_path),
                    });
                }
            }
            Err(doc_parser::ParseError::EmptyDocument) => {}
            Err(e) => {
                errors.push(format!("{}: {e}", path.display()));
            }
        }
    }

    (cards, errors)
}

#[cfg(feature = "web")]
fn setup_card_server(
    port: u16,
    deck_name: String,
    source_dirs: Vec<std::path::PathBuf>,
    recursive: bool,
    cwd: Option<std::path::PathBuf>,
) -> Result<()> {
    let static_root = std::env::current_dir().ok();
    let refresh = move || -> Result<web_preview::PreviewData, String> {
        let files = discover_card_files(&source_dirs, recursive)?;
        let (cards, errors) = parse_and_convert_cards(&files, &cwd);
        Ok(web_preview::PreviewData {
            deck_name: deck_name.clone(),
            cards,
            files_processed: files.len(),
            errors,
        })
    };
    web_preview::serve(port, static_root, refresh)
        .map_err(|e| anyhow!("Web server error: {}", e))
}

#[cfg(feature = "web")]
fn run_serve(deck: Option<String>, port: u16, no_recursive: bool) -> Result<()> {
    let cfg = load_config_with_banner()?;
    let deck_name = deck.or(cfg.deck).unwrap_or_else(|| "Preview".to_string());
    validate_deck_name(&deck_name)?;
    let recursive = !no_recursive && cfg.recursive.unwrap_or(true);
    let source_dirs = source_dir()?;
    let cwd = std::env::current_dir().ok();
    setup_card_server(port, deck_name, source_dirs, recursive, cwd)
}

#[cfg(not(feature = "web"))]
fn run_serve(_deck: Option<String>, _port: u16, _no_recursive: bool) -> Result<()> {
    bail!(
        "serve requires building with --features web.\n\
         Install with: cargo install acli --features web"
    )
}

// ── Init ──────────────────────────────────────────────────────────────────────

fn run_init(user: bool) -> Result<()> {
    if user {
        let target = config::user_config_path();
        if target.exists() {
            bail!("{} already exists", target.display());
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
            bail!(
                "{} already exists in the current directory",
                config::PROJECT_CONFIG_FILENAME
            );
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
) -> Result<()> {
    let anki_card_id = anki_wrapper::CardId(card_id);
    let anki_rating: anki_wrapper::ReviewRating = rating.into();

    #[cfg(feature = "real-anki")]
    {
        use anki_wrapper::{AnkiCollection, DefaultAnkiCollection};
        let mut collection = DefaultAnkiCollection::open_collection_path(collection_path)
            .map_err(|e| anyhow!("Failed to open Anki collection: {}", e))?;
        collection
            .record_review(anki_card_id, anki_rating)
            .map_err(|e| anyhow!("Failed to record review: {}", e))?;
        println!(
            "Recorded review for card {} with rating {:?}",
            card_id, anki_rating
        );
        Ok(())
    }

    #[cfg(not(feature = "real-anki"))]
    {
        let _ = (anki_card_id, anki_rating, collection_path);
        bail!("review-card requires the real-anki feature")
    }
}

fn run_get_reviews(card_id: i64, collection_path: &PathBuf) -> Result<()> {
    let anki_card_id = anki_wrapper::CardId(card_id);

    #[cfg(feature = "real-anki")]
    {
        use anki_wrapper::{AnkiCollection, DefaultAnkiCollection};
        let mut collection = DefaultAnkiCollection::open_collection_path(collection_path)
            .map_err(|e| anyhow!("Failed to open Anki collection: {}", e))?;
        let reviews = collection
            .get_reviews(anki_card_id)
            .map_err(|e| anyhow!("Failed to get reviews: {}", e))?;

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
                .context("Failed to serialize reviews to JSON")?
        );
        Ok(())
    }

    #[cfg(not(feature = "real-anki"))]
    {
        let _ = (anki_card_id, collection_path);
        bail!("get-reviews requires the real-anki feature")
    }
}
