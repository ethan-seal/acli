use std::fmt;

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub files_processed: usize,
    pub cards_synced: usize,
    pub deck_name: String,
}

impl fmt::Display for SyncResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sync Summary:\n  Files processed: {}\n  Cards synced: {}\n  Target deck: {}",
            self.files_processed, self.cards_synced, self.deck_name
        )
    }
}
