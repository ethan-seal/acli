# acli Implementation Plan

## Overview
The main CLI application that orchestrates document discovery, parsing, planning, and syncing to Anki. This is the user-facing component that ties all other components together.

## Core Architecture

```rust
use doc_parser::{DocumentParser, MarkdownParser};
use update_planner::{UpdatePlanner, SimplePlanner, PlanExecutor, DefaultExecutor, DocumentSet};
use anki_wrapper::{AnkiCollection, DefaultAnkiCollection, DeckConfig};

pub struct SyncConfig {
    pub source_dirs: Vec<PathBuf>,
    pub deck_name: String,
    pub anki_collection_path: Option<PathBuf>, // None = default
}

pub struct AnkiCli {
    parser: MarkdownParser,
    planner: SimplePlanner,
    executor: DefaultExecutor,
}
```

## CLI Interface

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "acli")]
#[command(about = "Sync markdown documents to Anki")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
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
        #[arg(short, long, default_value_t = true)]
        recursive: bool,
    },

    /// Validate markdown files without syncing
    Validate {
        /// Directory containing markdown files
        #[arg(short, long)]
        source: PathBuf,

        /// Recursive directory traversal
        #[arg(short, long, default_value_t = true)]
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
        #[arg(short, long, default_value_t = true)]
        recursive: bool,
    },
}
```

## Main Sync Logic

```rust
impl AnkiCli {
    pub fn sync(&self, config: &SyncConfig) -> Result<SyncResult, CliError> {
        // 1. Discover markdown files
        let markdown_files = self.discover_files(&config.source_dirs)?;
        eprintln!("Found {} markdown files", markdown_files.len());

        // 2. Parse all documents
        let mut all_cards = Vec::new();
        let mut source_files = Vec::new();

        for file_path in &markdown_files {
            let content = std::fs::read_to_string(file_path)?;

            match self.parser.parse(&content) {
                Ok(parsed_doc) => {
                    all_cards.extend(parsed_doc.cards);
                    source_files.push(file_path.to_string_lossy().to_string());
                    eprintln!("✓ Parsed {}: {} cards", file_path.display(), parsed_doc.cards.len());
                },
                Err(e) => {
                    eprintln!("✗ Failed to parse {}: {}", file_path.display(), e);
                    return Err(CliError::ParseError {
                        file: file_path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // 3. Create document set
        let document_set = DocumentSet {
            cards: all_cards,
            source_files,
        };

        eprintln!("Total cards to sync: {}", document_set.cards.len());

        // 4. Generate sync plan
        let plan = self.planner.plan_fresh_sync(&document_set, &config.deck_name)?;
        eprintln!("Generated sync plan with {} operations", plan.operations.len());

        // 5. Execute plan
        let mut collection = DefaultAnkiCollection::open(config.anki_collection_path.as_ref())?;
        self.executor.execute_plan(&plan, &mut collection)?;

        eprintln!("✓ Sync completed successfully");

        Ok(SyncResult {
            files_processed: markdown_files.len(),
            cards_synced: document_set.cards.len(),
            deck_name: config.deck_name.clone(),
        })
    }

    fn discover_files(&self, source_dirs: &[PathBuf]) -> Result<Vec<PathBuf>, CliError> {
        let mut files = Vec::new();

        for dir in source_dirs {
            if dir.is_file() && self.is_markdown_file(dir) {
                files.push(dir.clone());
                continue;
            }

            if !dir.is_dir() {
                return Err(CliError::InvalidPath(dir.clone()));
            }

            // Recursive directory traversal
            for entry in walkdir::WalkDir::new(dir) {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && self.is_markdown_file(path) {
                    files.push(path.to_path_buf());
                }
            }
        }

        files.sort();
        Ok(files)
    }

    fn is_markdown_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false)
    }
}
```

## Error Handling

```rust
#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Failed to parse file {file}: {error}")]
    ParseError {
        file: PathBuf,
        error: String,
    },

    #[error("Planning error: {0}")]
    PlanningError(#[from] update_planner::PlanError),

    #[error("Anki error: {0}")]
    AnkiError(#[from] anki_wrapper::AnkiWrapperError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Directory traversal error: {0}")]
    WalkdirError(#[from] walkdir::Error),
}
```

## Output and Reporting

```rust
#[derive(Debug)]
pub struct SyncResult {
    pub files_processed: usize,
    pub cards_synced: usize,
    pub deck_name: String,
}

impl std::fmt::Display for SyncResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sync Summary:\n  Files processed: {}\n  Cards synced: {}\n  Target deck: {}",
            self.files_processed, self.cards_synced, self.deck_name
        )
    }
}
```

## Dependencies
- `clap` - CLI argument parsing (feature = "derive")
- `walkdir` - Directory traversal
- `thiserror` - Error handling
- `doc-parser` (local crate)
- `update-planner` (local crate)
- `anki-wrapper` (local crate)

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_markdown_files() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        std::fs::write(&md_file, "# Test\nHello -> World").unwrap();

        let cli = AnkiCli::new();
        let files = cli.discover_files(&[temp_dir.path().to_path_buf()]).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], md_file);
    }

    #[test]
    fn test_markdown_file_detection() {
        let cli = AnkiCli::new();

        assert!(cli.is_markdown_file(Path::new("test.md")));
        assert!(cli.is_markdown_file(Path::new("test.markdown")));
        assert!(cli.is_markdown_file(Path::new("TEST.MD")));
        assert!(!cli.is_markdown_file(Path::new("test.txt")));
        assert!(!cli.is_markdown_file(Path::new("test")));
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use anki_wrapper::FakeAnkiCollection;

    #[test]
    fn test_end_to_end_sync() {
        // Create test markdown files
        let temp_dir = TempDir::new().unwrap();
        let md_content = r#"
# Math Cards

Basic addition:
- 2 + 2 -> 4
- 3 + 3 <-> 6

## Advanced
- calculus -> derivatives and integrals
"#;

        let md_file = temp_dir.path().join("math.md");
        std::fs::write(&md_file, md_content).unwrap();

        // Run sync with fake collection
        let cli = AnkiCli::new();
        let config = SyncConfig {
            source_dirs: vec![temp_dir.path().to_path_buf()],
            deck_name: "TestDeck".to_string(),
            anki_collection_path: None,
        };

        // This would use a test-specific version that uses FakeAnkiCollection
        let result = cli.sync_with_fake_collection(&config);
        assert!(result.is_ok());

        let sync_result = result.unwrap();
        assert_eq!(sync_result.files_processed, 1);
        assert_eq!(sync_result.cards_synced, 3);
    }
}
```

## CLI Examples

```bash
# Sync current directory to "Math" deck
acli sync --source . --deck "Math"

# Sync specific directory recursively
acli sync --source ~/notes/math --deck "Mathematics" --recursive

# Validate files without syncing
acli validate --source ~/notes

# Preview what would be synced
acli preview --source ~/notes/physics --deck "Physics"

# Use specific Anki collection
acli sync --source . --deck "Test" --collection ~/anki/User1/collection.anki2
```

## File Structure
```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library interface
├── cli.rs            # CLI argument parsing and commands
├── sync.rs           # Main sync logic
├── discovery.rs      # File discovery logic
├── error.rs          # Error types
└── output.rs         # Output formatting and reporting
tests/
├── integration.rs    # End-to-end tests
├── cli.rs           # CLI interface tests
└── discovery.rs     # File discovery tests
examples/
└── basic_usage.rs   # Example usage as library
```

## Implementation Steps
1. Set up Cargo.toml with dependencies
2. Implement CLI argument parsing with clap
3. Create file discovery logic
4. Implement main sync orchestration
5. Add comprehensive error handling
6. Create output formatting and progress reporting
7. Write extensive test suite
8. Add example usage and documentation
9. Integration testing with all components

## User Experience Considerations
- Clear progress reporting during sync
- Helpful error messages with file context
- Dry-run/preview mode for safety
- Validation mode for checking syntax
- Configurable verbosity levels
- Graceful handling of permission errors

## Future Enhancements
- Configuration file support (acli.toml)
- Watch mode for continuous syncing
- Selective file filtering (ignore patterns)
- Parallel processing for large document sets
- Backup/restore functionality
- Statistics and analytics
- Plugin system for custom card types