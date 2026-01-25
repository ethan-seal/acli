use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

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
    }
}
