use crate::types::{Card, CardType};

use super::extraction::{build_residual, ExtractionResult};

/// Scan `markdown` for sequence blocks (`=>` prefixed lines) and extract them
/// as `Card` values, returning them alongside the residual text with those
/// lines removed.
///
/// A sequence block is a contiguous run of lines starting with `=> ` (or `=>`)
/// that is immediately preceded by a plain-text label line (non-empty, not
/// starting with `=>` or common Markdown block-level characters).
pub(super) fn extract_sequence_cards(markdown: &str) -> ExtractionResult {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards: Vec<Card> = Vec::new();
    let mut consumed: Vec<bool> = vec![false; lines.len()];

    let mut i = 0;
    while i < lines.len() {
        if is_sequence_line(lines[i]) {
            let block_start = i;
            while i < lines.len() && is_sequence_line(lines[i]) {
                i += 1;
            }
            let block_end = i; // exclusive

            let label_opt = find_label(&lines, block_start);

            if let Some((label_idx, label)) = label_opt {
                let steps: Vec<&str> = lines[block_start..block_end]
                    .iter()
                    .map(|l| strip_sequence_prefix(l))
                    .collect();

                cards.extend(make_sequence_cards(&label, &steps));

                consumed[label_idx] = true;
                for item in consumed.iter_mut().take(block_end).skip(block_start) {
                    *item = true;
                }
            }
        } else {
            i += 1;
        }
    }

    ExtractionResult {
        cards,
        residual: build_residual(&lines, &consumed),
        warnings: Vec::new(),
    }
}

/// Returns true if `line` begins a sequence step (`=> ` or `=>`).
fn is_sequence_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("=> ") || trimmed == "=>"
}

/// Strip the `=> ` prefix and return the step text.
fn strip_sequence_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("=> ") {
        rest.trim()
    } else {
        trimmed.strip_prefix("=>").unwrap_or(trimmed).trim()
    }
}

/// Walk upward from `block_start` to find the label line.
///
/// The label is the first non-empty line immediately above the block that is
/// not itself a sequence line and does not look like a Markdown heading, list
/// item, or code-fence.
fn find_label(lines: &[&str], block_start: usize) -> Option<(usize, String)> {
    if block_start == 0 {
        return None;
    }
    let mut idx = block_start - 1;
    loop {
        let line = lines[idx].trim();
        if !line.is_empty() {
            if is_sequence_line(lines[idx]) {
                return None;
            }
            // Reject Markdown structural lines.
            if line.starts_with('#')
                || line.starts_with('-')
                || line.starts_with('*')
                || line.starts_with('>')
                || line.starts_with("```")
                || line.starts_with("~~~")
            {
                return None;
            }
            return Some((idx, line.to_string()));
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    None
}

/// Build the chain of `Sequence` cards from a label and an ordered list of steps.
fn make_sequence_cards(label: &str, steps: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let front = if i == 0 {
            format!("{}\nFirst:", label)
        } else {
            format!("{}\nAfter: {}", label, steps[i - 1])
        };
        cards.push(Card {
            card_type: CardType::Sequence,
            fields: vec![front, step.to_string()],
        });
    }
    cards
}
