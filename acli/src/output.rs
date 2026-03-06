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

    pub fn print_validation_success(&self, result: &crate::sync::ValidationResult) {
        if result.total_files == 0 {
            eprintln!("warning: no markdown files found in the current directory");
            return;
        }
        let skipped = result.total_files - result.files_with_cards;
        if skipped > 0 {
            println!(
                "Validation passed: {} files with cards, {} skipped",
                result.files_with_cards, skipped
            );
        } else {
            println!("Validation passed: {} files", result.total_files);
        }
    }

    pub fn print_preview(&self, result: &crate::sync::PreviewResult) {
        println!(
            "Preview: {} cards from {} files -> deck \"{}\"",
            result.cards.len(),
            result.files_processed,
            result.deck_name,
        );
        if result.cards.is_empty() {
            return;
        }
        println!();
        for (i, card) in result.cards.iter().enumerate() {
            let type_label = match card.card_type {
                doc_parser::CardType::Basic => "basic",
                doc_parser::CardType::Bidirectional => "bidi",
                doc_parser::CardType::Sequence => "sequence",
            };
            // Show front (first field), truncated to one line.
            let front = card.fields.first().map(|s| s.as_str()).unwrap_or("");
            let front_line = first_line(front);
            // Show back (second field), truncated to one line.
            let back = card.fields.get(1).map(|s| s.as_str()).unwrap_or("");
            let back_line = first_line(back);

            println!(
                "  {:>3}. [{}] {} -> {}",
                i + 1,
                type_label,
                front_line,
                back_line
            );
        }
    }

    pub fn print_error(&self, error: &dyn std::error::Error) {
        eprintln!("{error}");
    }
}

/// Extract the first line of a string, truncating to 60 chars with "..." if needed.
fn first_line(s: &str) -> String {
    let line = s.lines().next().unwrap_or("");
    if line.len() > 60 {
        format!("{}...", &line[..57])
    } else if s.lines().count() > 1 {
        format!("{} ...", line)
    } else {
        line.to_string()
    }
}
