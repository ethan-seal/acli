use crate::types::{Card, CardType};

use super::extraction::{build_residual, ExtractionResult};

/// Arrow marker kinds that appear on column headers.
#[derive(Debug, Clone, PartialEq)]
enum ArrowMarker {
    /// `<->` — emit both a forward and a reverse card.
    Bidirectional,
    /// `->` — emit only a forward card (row → cell).
    Forward,
    /// `<-` — emit only a reverse card (cell → row).
    Backward,
}

/// Suffix→marker mapping for `parse_col_header`, checked in order.
///
/// Order matters: `<->` must be checked before `<-` so the longer
/// marker takes priority.
const ARROW_SUFFIXES: &[(&str, ArrowMarker)] = &[
    (" <->", ArrowMarker::Bidirectional),
    ("<->", ArrowMarker::Bidirectional),
    (" ->", ArrowMarker::Forward),
    ("->", ArrowMarker::Forward),
    (" <-", ArrowMarker::Backward),
    ("<-", ArrowMarker::Backward),
];

/// A parsed column header: display name and optional arrow marker.
#[derive(Debug, Clone)]
struct ColHeader {
    name: String,
    arrow: Option<ArrowMarker>,
}

/// Parse a raw header cell string (already trimmed) into a [`ColHeader`].
///
/// Strips a trailing `<->`, `->`, or `<-` marker and returns the display name
/// alongside the marker.
fn parse_col_header(raw: &str) -> ColHeader {
    let raw = raw.trim();
    for (suffix, marker) in ARROW_SUFFIXES {
        if let Some(name) = raw.strip_suffix(suffix).map(str::trim) {
            return ColHeader {
                name: name.to_string(),
                arrow: Some(marker.clone()),
            };
        }
    }
    ColHeader {
        name: raw.to_string(),
        arrow: None,
    }
}

/// Split a raw pipe-separated line into trimmed cell strings.
///
/// Leading/trailing `|` characters are stripped before splitting so that both
/// `a | b | c` and `| a | b | c |` produce the same `["a", "b", "c"]` slice.
pub(super) fn split_pipe_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed.strip_prefix('|').unwrap_or(trimmed));
    inner.split('|').map(str::trim).collect()
}

/// Returns `true` if `line` is a pipe-table line (contains at least one `|`).
fn is_pipe_line(line: &str) -> bool {
    line.contains('|')
}

/// Emit forward and/or reverse cards for a single pipe-table cell.
fn emit_pipe_cards(
    cards: &mut Vec<Card>,
    arrow: &ArrowMarker,
    context: &str,
    row_value: &str,
    cell_value: &str,
) {
    if matches!(arrow, ArrowMarker::Bidirectional | ArrowMarker::Forward) {
        cards.push(Card {
            card_type: CardType::Basic,
            fields: vec![
                format!("{}\n{} -> ?", context, row_value),
                cell_value.to_string(),
            ],
        });
    }
    if matches!(arrow, ArrowMarker::Bidirectional | ArrowMarker::Backward) {
        cards.push(Card {
            card_type: CardType::Basic,
            fields: vec![
                format!("{}\n? <- {}", context, cell_value),
                row_value.to_string(),
            ],
        });
    }
}

/// Scan `markdown` for pipe-block groups and extract them as [`Card`] values.
///
/// A pipe block is a contiguous run of lines that each contain `|`.  The first
/// line of the block is treated as the header row; subsequent lines are data
/// rows.
///
/// Returns the extracted cards and the residual text with those lines removed.
pub(super) fn extract_pipe_table_cards(markdown: &str) -> ExtractionResult {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards: Vec<Card> = Vec::new();
    let mut consumed: Vec<bool> = vec![false; lines.len()];

    let mut i = 0;
    while i < lines.len() {
        if is_pipe_line(lines[i]) {
            // Find the full contiguous pipe block.
            let block_start = i;
            while i < lines.len() && is_pipe_line(lines[i]) {
                i += 1;
            }
            let block_end = i; // exclusive

            // At least two lines needed (header + one data row).
            if block_end - block_start < 2 {
                continue;
            }

            // Parse header row.
            let header_cells = split_pipe_cells(lines[block_start]);
            if header_cells.is_empty() {
                continue;
            }

            // The subject column is always the first column (index 0).
            let subject_header = header_cells[0].trim().to_string();

            // Parse remaining header columns.
            let value_headers: Vec<ColHeader> = header_cells[1..]
                .iter()
                .map(|c| parse_col_header(c))
                .collect();

            // Skip tables where no value column has an arrow marker.
            let any_arrow = value_headers.iter().any(|h| h.arrow.is_some());
            if !any_arrow {
                continue;
            }

            // Emit cards for each data row.
            for row_line in &lines[block_start + 1..block_end] {
                let cells = split_pipe_cells(row_line);
                if cells.is_empty() {
                    continue;
                }

                let row_value = cells[0].trim();
                if row_value.is_empty() {
                    continue;
                }

                for (col_idx, header) in value_headers.iter().enumerate() {
                    let arrow = match &header.arrow {
                        Some(a) => a,
                        None => continue,
                    };

                    let cell_value = cells
                        .get(col_idx + 1)
                        .map(|s| s.trim())
                        .unwrap_or("");

                    if cell_value.is_empty() {
                        continue;
                    }

                    let context = format!("{} -> {}", subject_header, header.name);
                    emit_pipe_cards(&mut cards, arrow, &context, row_value, cell_value);
                }
            }

            // Mark all block lines as consumed.
            for item in consumed.iter_mut().take(block_end).skip(block_start) {
                *item = true;
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
