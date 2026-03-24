use crate::types::{Card, CardType, Warning};

use super::extraction::{build_residual, ExtractionResult};

/// Scan `markdown` for block cards (a `->` or `<->` on its own line separating
/// question content above from answer content below) and extract them as `Card`
/// values, returning them alongside the residual text with consumed lines
/// removed.
///
/// A block card is:
/// - Contiguous non-blank lines above the arrow (the question)
/// - A line containing only `->` or `<->` (the separator)
/// - Contiguous non-blank lines below the arrow (the answer)
///
/// Blank lines delimit the question and answer blocks.  Both sides must have
/// at least one line of content.
pub(super) fn extract_block_cards(markdown: &str) -> ExtractionResult {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards: Vec<Card> = Vec::new();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut consumed: Vec<bool> = vec![false; lines.len()];

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_block_arrow = trimmed == "->" || trimmed == "<->";
        if !is_block_arrow {
            i += 1;
            continue;
        }

        let is_bidi = trimmed == "<->";
        let arrow_idx = i;
        let line_num = arrow_idx + 1;

        // Walk backward from the arrow to find question content.
        let mut q_start = arrow_idx;
        while q_start > 0 {
            let prev = q_start - 1;
            if lines[prev].trim().is_empty() || consumed[prev] {
                break;
            }
            q_start = prev;
        }

        // Walk forward from the arrow to find answer content.
        let mut a_end = arrow_idx + 1;
        while a_end < lines.len() {
            if lines[a_end].trim().is_empty() || consumed[a_end] {
                break;
            }
            a_end += 1;
        }

        let has_question = q_start < arrow_idx;
        let has_answer = arrow_idx + 1 < a_end;

        if has_question && has_answer {
            let question = lines[q_start..arrow_idx].join("\n");
            let answer = lines[arrow_idx + 1..a_end].join("\n");

            let card_type = if is_bidi {
                CardType::Bidirectional
            } else {
                CardType::Basic
            };

            cards.push(Card {
                card_type,
                fields: vec![question, answer],
            });

            for item in consumed.iter_mut().take(a_end).skip(q_start) {
                *item = true;
            }
        } else {
            let arrow_str = if is_bidi { "<->" } else { "->" };
            let missing = if !has_question && !has_answer {
                "missing question and answer"
            } else if !has_question {
                "missing question above arrow"
            } else {
                "missing answer below arrow"
            };
            warnings.push(Warning::new(
                line_num,
                format!("block card `{}` {}", arrow_str, missing),
            ));
            consumed[arrow_idx] = true;
        }

        i = a_end.max(arrow_idx + 1);
    }

    ExtractionResult {
        cards,
        residual: build_residual(&lines, &consumed),
        warnings,
    }
}
