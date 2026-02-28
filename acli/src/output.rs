use std::fmt;

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub files_processed: usize,
    pub cards_synced: usize,
    pub cards_added: usize,
    pub cards_deleted: usize,
    pub cards_unchanged: usize,
    pub deck_name: String,
    pub dry_run: bool,
    pub media_copied: usize,
    pub media_skipped: usize,
    pub media_missing: usize,
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
            cards_added: 0,
            cards_deleted: 0,
            cards_unchanged: 0,
            deck_name,
            dry_run,
            media_copied: 0,
            media_skipped: 0,
            media_missing: 0,
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
        )?;
        if !self.dry_run {
            write!(
                f,
                "\n  Cards added: {}\n  Cards deleted: {}\n  Cards unchanged: {}",
                self.cards_added, self.cards_deleted, self.cards_unchanged
            )?;
        }
        let total_media = self.media_copied + self.media_skipped + self.media_missing;
        if total_media > 0 {
            write!(
                f,
                "\n  Media copied: {}\n  Media skipped: {}\n  Media missing: {}",
                self.media_copied, self.media_skipped, self.media_missing
            )?;
        }
        Ok(())
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
