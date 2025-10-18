use std::fmt;

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub files_processed: usize,
    pub cards_synced: usize,
    pub deck_name: String,
    pub dry_run: bool,
}

impl SyncResult {
    pub fn new(
        files_processed: usize,
        cards_synced: usize,
        deck_name: String,
        dry_run: bool,
    ) -> Self {
        Self {
            files_processed,
            cards_synced,
            deck_name,
            dry_run,
        }
    }
}

impl fmt::Display for SyncResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sync Summary:\n  Files processed: {}\n  Cards synced: {}\n  Target deck: {}\n  Dry run: {}",
            self.files_processed,
            self.cards_synced,
            self.deck_name,
            if self.dry_run { "yes" } else { "no" }
        )
    }
}

#[derive(Debug, Default)]
pub struct CliOutput;

impl CliOutput {
    pub fn print_sync_result(&self, result: &SyncResult) {
        println!("{result}");
    }

    pub fn print_validation_success(&self, files_checked: usize) {
        println!("✓ Validation successful ({files_checked} files)");
    }

    pub fn print_error(&self, error: &dyn std::error::Error) {
        eprintln!("{error}");
    }
}
