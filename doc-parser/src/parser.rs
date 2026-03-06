//! Parser traits and Markdown parser implementation.
use crate::error::ParseError;
use crate::types::{Card, CardType, MediaReference, ParsedDocument};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::HashSet;

// ── Template pre-processor ───────────────────────────────────────────────────

/// Expand template blocks in the markdown.
///
/// A template block is a code fence with `template` language followed by a
/// pipe table.  Each data row is expanded through the template using simple
/// `{{ column }}` substitution, and the template+table is replaced with the
/// expanded text.
///
/// Returns the markdown with all template blocks expanded.  Non-template
/// content passes through unchanged.
fn expand_templates(markdown: &str) -> Result<String, ParseError> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Look for a template code fence opening.
        let trimmed = lines[i].trim();
        if trimmed == "```template" || trimmed == "~~~template" {
            let fence_char = if trimmed.starts_with('`') { '`' } else { '~' };
            let fence_open = i;
            i += 1;

            // Collect template body until closing fence.
            let mut template_lines: Vec<&str> = Vec::new();
            let mut fence_close = None;
            while i < lines.len() {
                let t = lines[i].trim();
                if (fence_char == '`' && t.starts_with("```"))
                    || (fence_char == '~' && t.starts_with("~~~"))
                {
                    fence_close = Some(i);
                    i += 1;
                    break;
                }
                template_lines.push(lines[i]);
                i += 1;
            }

            let fence_close = match fence_close {
                Some(fc) => fc,
                None => {
                    // Unclosed fence — pass through as-is.
                    for line in &lines[fence_open..] {
                        result_lines.push(line.to_string());
                    }
                    break;
                }
            };

            let template = template_lines.join("\n");

            // Skip blank lines between the closing fence and the pipe table.
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }

            // Collect the pipe table (header + data rows).
            if i >= lines.len() || !lines[i].contains('|') {
                // No pipe table after template — pass through as-is.
                for line in &lines[fence_open..=fence_close] {
                    result_lines.push(line.to_string());
                }
                continue;
            }

            // Parse header row.
            let header_cells: Vec<&str> = split_pipe_cells(lines[i])
                .iter()
                .map(|s| s.trim())
                .collect::<Vec<_>>()
                .into_iter()
                .collect();
            let header_names: Vec<String> = header_cells.iter().map(|s| s.to_string()).collect();
            i += 1;

            // Parse data rows.
            let mut data_rows: Vec<Vec<String>> = Vec::new();
            while i < lines.len() && lines[i].contains('|') {
                let cells: Vec<String> = split_pipe_cells(lines[i])
                    .iter()
                    .map(|s| s.trim().to_string())
                    .collect();
                data_rows.push(cells);
                i += 1;
            }

            // Expand template for each row.
            for (row_idx, row) in data_rows.iter().enumerate() {
                // Check for empty or missing cells — every placeholder must
                // have a value.
                for (col_idx, col_name) in header_names.iter().enumerate() {
                    let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                    if cell.is_empty() {
                        return Err(ParseError::TemplateError(format!(
                            "empty cell for column '{}' in row {} of template at line {}",
                            col_name,
                            row_idx + 1,
                            fence_open + 1,
                        )));
                    }
                }

                let expanded = expand_template_row(&template, &header_names, row, fence_open)?;

                // Add blank line between expanded rows for block card separation.
                if row_idx > 0 {
                    result_lines.push(String::new());
                }
                for line in expanded.lines() {
                    result_lines.push(line.to_string());
                }
            }
        } else {
            result_lines.push(lines[i].to_string());
            i += 1;
        }
    }

    Ok(result_lines.join("\n"))
}

/// Expand a single template row by replacing `{{ column }}` placeholders.
///
/// Handles optional whitespace inside the braces: `{{name}}`, `{{ name }}`,
/// `{{ name}}`, etc.  Reports an error for unknown placeholders.
fn expand_template_row(
    template: &str,
    header_names: &[String],
    row: &[String],
    fence_open: usize,
) -> Result<String, ParseError> {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        // Copy everything before the placeholder.
        result.push_str(&rest[..start]);

        let after_open = &rest[start + 2..];
        match after_open.find("}}") {
            Some(end) => {
                let name = after_open[..end].trim();
                // Look up the column name.
                let col_idx = header_names.iter().position(|h| h == name);
                match col_idx {
                    Some(idx) => {
                        let value = row.get(idx).map(|s| s.as_str()).unwrap_or("");
                        result.push_str(value);
                    }
                    None => {
                        return Err(ParseError::TemplateError(format!(
                            "unknown placeholder '{}' in template at line {}",
                            name,
                            fence_open + 1,
                        )));
                    }
                }
                rest = &after_open[end + 2..];
            }
            None => {
                // No closing `}}` — copy literally and stop.
                result.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    // Copy any remaining text after the last placeholder.
    result.push_str(rest);

    Ok(result)
}

// ── Pipe-table pre-processor ─────────────────────────────────────────────────

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
    // Order matters: check `<->` before `<-`.
    if let Some(name) = raw.strip_suffix(" <->").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Bidirectional),
        };
    }
    if let Some(name) = raw.strip_suffix("<->").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Bidirectional),
        };
    }
    if let Some(name) = raw.strip_suffix(" ->").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Forward),
        };
    }
    if let Some(name) = raw.strip_suffix("->").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Forward),
        };
    }
    // Check `<-` last so it doesn't shadow `<->`.
    if let Some(name) = raw.strip_suffix(" <-").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Backward),
        };
    }
    if let Some(name) = raw.strip_suffix("<-").map(str::trim) {
        return ColHeader {
            name: name.to_string(),
            arrow: Some(ArrowMarker::Backward),
        };
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
fn split_pipe_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim();
    // Strip optional surrounding pipes.
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

/// Scan `markdown` for pipe-block groups and extract them as [`Card`] values.
///
/// A pipe block is a contiguous run of lines that each contain `|`.  The first
/// line of the block is treated as the header row; subsequent lines are data
/// rows.
///
/// Returns the extracted cards and the residual text with those lines removed.
fn extract_pipe_table_cards(markdown: &str) -> (Vec<Card>, String) {
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
                // Skip single-line blocks — leave them un-consumed.
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

                // Row value is always the first cell.
                let row_value = cells[0].trim();
                if row_value.is_empty() {
                    continue;
                }

                // For each value column with an arrow marker…
                for (col_idx, header) in value_headers.iter().enumerate() {
                    let arrow = match &header.arrow {
                        Some(a) => a,
                        None => continue, // display-only column
                    };

                    // Cell value — may be missing (empty trailing cell).
                    let cell_value = cells
                        .get(col_idx + 1) // +1 because cells[0] is the subject
                        .map(|s| s.trim())
                        .unwrap_or("");

                    if cell_value.is_empty() {
                        // No card for empty cells.
                        continue;
                    }

                    // Context string: "<subject col> → <value col>"
                    let context = format!("{} \u{2192} {}", subject_header, header.name);

                    // Forward card: front = "<context>\n<row value> →?" / back = cell
                    if matches!(arrow, ArrowMarker::Bidirectional | ArrowMarker::Forward) {
                        let front = format!("{}\n{} \u{2192}?", context, row_value);
                        cards.push(Card {
                            card_type: CardType::Basic,
                            fields: vec![front, cell_value.to_string()],
                        });
                    }

                    // Reverse card: front = "<context>\n? ← <cell>" / back = row value
                    if matches!(arrow, ArrowMarker::Bidirectional | ArrowMarker::Backward) {
                        let front = format!("{}\n? \u{2190} {}", context, cell_value);
                        cards.push(Card {
                            card_type: CardType::Basic,
                            fields: vec![front, row_value.to_string()],
                        });
                    }
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

    // Build residual text from un-consumed lines.
    let residual: String = lines
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed[*idx])
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n");

    (cards, residual)
}

// ── Sequence pre-processor ────────────────────────────────────────────────────

/// Scan `markdown` for sequence blocks (`=>` prefixed lines) and extract them
/// as `Card` values, returning them alongside the residual text with those
/// lines removed.
///
/// A sequence block is a contiguous run of lines starting with `=> ` (or `=>`)
/// that is immediately preceded by a plain-text label line (non-empty, not
/// starting with `=>` or common Markdown block-level characters).
fn extract_sequence_cards(markdown: &str) -> (Vec<Card>, String) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards: Vec<Card> = Vec::new();
    // Track which line indices are consumed by sequence blocks (including labels).
    let mut consumed: Vec<bool> = vec![false; lines.len()];

    let mut i = 0;
    while i < lines.len() {
        // Look for the start of a `=>` block.
        if is_sequence_line(lines[i]) {
            // Find the contiguous block of `=>` lines.
            let block_start = i;
            while i < lines.len() && is_sequence_line(lines[i]) {
                i += 1;
            }
            let block_end = i; // exclusive

            // Find the label: the nearest non-empty line immediately above the block.
            let label_opt = find_label(&lines, block_start);

            if let Some((label_idx, label)) = label_opt {
                // Collect step texts.
                let steps: Vec<&str> = lines[block_start..block_end]
                    .iter()
                    .map(|l| strip_sequence_prefix(l))
                    .collect();

                // Emit cards.
                cards.extend(make_sequence_cards(&label, &steps));

                // Mark label + all sequence lines as consumed.
                consumed[label_idx] = true;
                for item in consumed.iter_mut().take(block_end).skip(block_start) {
                    *item = true;
                }
            }
            // If no label found, leave lines un-consumed (they'll pass through as text).
        } else {
            i += 1;
        }
    }

    // Build residual text from un-consumed lines.
    let residual: String = lines
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed[*idx])
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n");

    (cards, residual)
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
    // Search backwards, skipping blank lines, for the immediate preceding text line.
    let mut idx = block_start - 1;
    loop {
        let line = lines[idx].trim();
        if !line.is_empty() {
            // Must be a plain-text line — not a sequence line, heading, list marker, etc.
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

// ── Block-card pre-processor ──────────────────────────────────────────────────

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
fn extract_block_cards(markdown: &str) -> (Vec<Card>, String) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut cards: Vec<Card> = Vec::new();
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

        // Walk backward from the arrow to find question content.
        // Stop at a blank line (or already-consumed line, or start of file).
        let mut q_start = arrow_idx;
        while q_start > 0 {
            let prev = q_start - 1;
            if lines[prev].trim().is_empty() || consumed[prev] {
                break;
            }
            q_start = prev;
        }

        // Walk forward from the arrow to find answer content.
        // Stop at a blank line (or already-consumed line, or end of file).
        let mut a_end = arrow_idx + 1;
        while a_end < lines.len() {
            if lines[a_end].trim().is_empty() || consumed[a_end] {
                break;
            }
            a_end += 1;
        }

        // Both sides must have at least one line of content.
        if q_start < arrow_idx && arrow_idx + 1 < a_end {
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

            // Mark all consumed lines (question + arrow + answer).
            for idx in q_start..a_end {
                consumed[idx] = true;
            }
        } else {
            // Incomplete block card (missing question or answer).
            // Consume the arrow line so it doesn't leak through to
            // pulldown-cmark and create a spurious inline card.
            consumed[arrow_idx] = true;
        }

        i = a_end.max(arrow_idx + 1);
    }

    // Build residual text from un-consumed lines.
    let residual: String = lines
        .iter()
        .enumerate()
        .filter(|(idx, _)| !consumed[*idx])
        .map(|(_, l)| *l)
        .collect::<Vec<_>>()
        .join("\n");

    (cards, residual)
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

/// Trait for parsing documents into cards.
pub trait DocumentParser {
    /// Error type returned by the parser.
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Parse a Markdown string into a parsed document containing cards.
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error>;
}

/// Parser that recognises simple `->` and `<->` card syntax in Markdown.
pub struct MarkdownParser;

impl MarkdownParser {
    /// Create a new Markdown parser instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

// ── Span ──────────────────────────────────────────────────────────────────────

/// A single inline content unit within a list item or paragraph.
#[derive(Clone, Debug)]
enum Span {
    /// Plain text — may contain `->` / `<->` delimiters.
    Text(String),
    /// Inline code content — arrows here are **never** treated as delimiters.
    Code(String),
    /// An inline image, rendered as an `<img>` tag in field output.
    Image { url: String, alt: String },
}

/// Render a span sequence to a plain string suitable for a card field.
///
/// `Code` content is emitted as-is; `Image` spans become `<img>` tags.
fn render(spans: &[Span]) -> String {
    spans.iter().fold(String::new(), |mut out, span| {
        match span {
            Span::Text(s) => out.push_str(s),
            Span::Code(s) => out.push_str(s),
            Span::Image { url, alt } => out.push_str(&format!(r#"<img src="{url}" alt="{alt}">"#)),
        }
        out
    })
}

/// Append plain text to a span buffer, coalescing adjacent `Text` nodes.
fn push_text(spans: &mut Vec<Span>, s: &str) {
    match spans.last_mut() {
        Some(Span::Text(t)) => t.push_str(s),
        _ => spans.push(Span::Text(s.to_string())),
    }
}

/// Split `spans` at the first occurrence of `arrow` inside a `Span::Text` node.
///
/// `Span::Code` and `Span::Image` nodes are skipped, so arrows in inline code
/// (e.g. `` `->` ``) are never treated as delimiters.
///
/// Returns `(lhs, rhs)` with the arrow token itself consumed and edges trimmed.
fn split_at_arrow(spans: &[Span], arrow: &str) -> Option<(Vec<Span>, Vec<Span>)> {
    for (i, span) in spans.iter().enumerate() {
        if let Span::Text(t) = span {
            if let Some(pos) = t.find(arrow) {
                let lhs_tail = t[..pos].trim_end().to_string();
                let rhs_head = t[pos + arrow.len()..].trim_start().to_string();

                let mut lhs = spans[..i].to_vec();
                if !lhs_tail.is_empty() {
                    lhs.push(Span::Text(lhs_tail));
                }

                let mut rhs = Vec::new();
                if !rhs_head.is_empty() {
                    rhs.push(Span::Text(rhs_head));
                }
                rhs.extend_from_slice(&spans[i + 1..]);

                return Some((lhs, rhs));
            }
        }
    }
    None
}

// ── Accumulator types ─────────────────────────────────────────────────────────

/// Inline content being built for the current list item.
struct ItemAccum {
    spans: Vec<Span>,
    /// Set once a nested `Start(List)` fires — marks this item as a context
    /// parent rather than a leaf card.
    has_sublist: bool,
    /// Value of `list_depth` when `Start(Item)` fired for this item.
    depth: usize,
}

impl ItemAccum {
    fn new(depth: usize) -> Self {
        Self {
            spans: Vec::new(),
            has_sublist: false,
            depth,
        }
    }
}

/// Alt-text being collected between `Start(Image)` and `End(Image)`.
struct ImageAccum {
    url: String,
    alt: String,
}

// ── Parser state ──────────────────────────────────────────────────────────────

/// All mutable state for one `parse()` call, with one method per event type.
struct ParseState {
    list_depth: usize,
    /// `(item_depth, rendered_text)` pairs for ancestor context items.
    context_stack: Vec<(usize, String)>,
    /// `Some` while inside a list item.
    current_item: Option<ItemAccum>,
    /// `Some` while collecting an inline image's alt text.
    current_image: Option<ImageAccum>,
    /// `Some` while inside a top-level (non-list) paragraph.
    paragraph: Option<Vec<Span>>,
    /// Text accumulator while inside a heading element.
    heading_accum: Option<String>,
    /// The most recently seen heading text; used as subject label for attribute cards.
    current_heading: Option<String>,
}

impl ParseState {
    fn new() -> Self {
        Self {
            list_depth: 0,
            context_stack: Vec::new(),
            current_item: None,
            current_image: None,
            paragraph: None,
            heading_accum: None,
            current_heading: None,
        }
    }

    // ── Routing helper ────────────────────────────────────────────────────────

    /// Return the span buffer that should receive the next text fragment,
    /// or `None` if we're not currently inside any accumulator.
    ///
    /// Image alt-text takes priority over item/paragraph text so that text
    /// events fired between `Start(Image)` and `End(Image)` go to the alt field.
    fn text_sink(&mut self) -> Option<&mut Vec<Span>> {
        if self.current_image.is_some() {
            // Image alt is a plain String, not Vec<Span>; handled separately.
            return None;
        }
        if let Some(item) = self.current_item.as_mut() {
            return Some(&mut item.spans);
        }
        self.paragraph.as_mut()
    }

    /// Return the span buffer for break events (soft/hard).
    ///
    /// Breaks are *not* routed into image alt-text.
    fn break_sink(&mut self) -> Option<&mut Vec<Span>> {
        if let Some(item) = self.current_item.as_mut() {
            return Some(&mut item.spans);
        }
        self.paragraph.as_mut()
    }

    // ── Event handlers ────────────────────────────────────────────────────────

    fn on_start_list(&mut self) {
        self.list_depth += 1;
        if let Some(item) = self.current_item.as_mut() {
            // This item contains a sub-list — promote it to a context parent.
            item.has_sublist = true;
            let text = render(&item.spans).trim().to_string();
            self.context_stack.push((item.depth, text));
            item.spans.clear();
        }
    }

    fn on_end_list(&mut self) {
        self.list_depth -= 1;
        // Drop context entries that belonged to the list level we just left.
        while self
            .context_stack
            .last()
            .is_some_and(|(d, _)| *d > self.list_depth)
        {
            self.context_stack.pop();
        }
    }

    fn on_start_item(&mut self) {
        self.current_item = Some(ItemAccum::new(self.list_depth));
    }

    /// Consumes the current item and returns a `Card` if it is a leaf item
    /// with an arrow delimiter; returns `None` for context parents or items
    /// without a recognisable delimiter.
    fn on_end_item(&mut self) -> Option<Card> {
        let item = self.current_item.take()?;
        if item.has_sublist {
            return None; // Context parent — not a leaf card.
        }
        make_card(
            &item.spans,
            &self.context_stack,
            item.depth,
            self.current_heading.as_deref(),
        )
    }

    fn on_start_image(&mut self, url: String) {
        self.current_image = Some(ImageAccum {
            url,
            alt: String::new(),
        });
    }

    /// Finalises the current image accumulator, pushing an `Image` span into
    /// the active buffer and returning `(url, alt)` so the caller can create a
    /// `MediaReference`.  Returns `None` if no image was being accumulated.
    fn on_end_image(&mut self) -> Option<(String, String)> {
        let img = self.current_image.take()?;
        let span = Span::Image {
            url: img.url.clone(),
            alt: img.alt.clone(),
        };
        if let Some(item) = self.current_item.as_mut() {
            item.spans.push(span);
        } else if let Some(para) = self.paragraph.as_mut() {
            para.push(span);
        }
        Some((img.url, img.alt))
    }

    fn on_start_paragraph(&mut self) {
        if self.list_depth == 0 {
            self.paragraph = Some(Vec::new());
        }
    }

    fn on_end_paragraph(&mut self) -> Option<Card> {
        let spans = self.paragraph.take()?;
        make_card(&spans, &[], 0, None)
    }

    fn on_text(&mut self, s: &str) {
        if let Some(img) = self.current_image.as_mut() {
            img.alt.push_str(s);
        } else if let Some(h) = self.heading_accum.as_mut() {
            h.push_str(s);
        } else if let Some(buf) = self.text_sink() {
            push_text(buf, s);
        }
    }

    fn on_code_span(&mut self, s: &str) {
        if let Some(buf) = self.break_sink() {
            buf.push(Span::Code(s.to_string()));
        }
    }

    fn on_soft_break(&mut self) {
        if let Some(buf) = self.break_sink() {
            push_text(buf, " ");
        }
    }

    fn on_hard_break(&mut self) {
        if let Some(buf) = self.break_sink() {
            push_text(buf, "\n");
        }
    }

    fn on_start_heading(&mut self) {
        self.heading_accum = Some(String::new());
    }

    fn on_end_heading(&mut self) {
        if let Some(text) = self.heading_accum.take() {
            let trimmed = text.trim().to_string();
            self.current_heading = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
        }
    }
}

// ── Card construction ─────────────────────────────────────────────────────────

/// Attempt to build a [`Card`] from the accumulated spans of a list item or
/// paragraph. Tries `<->` before `->` so bidirectional cards are preferred.
///
/// When `heading` is `Some` and the item uses `->` syntax (attribute pattern),
/// the card front is formatted as `"{heading}\n{key} →?"` (two lines).
/// `<->` items are never treated as attribute cards.
fn make_card(
    spans: &[Span],
    context: &[(usize, String)],
    depth: usize,
    heading: Option<&str>,
) -> Option<Card> {
    // Try bidirectional first — heading context does not apply to <-> cards.
    if let Some((lhs, rhs)) = split_at_arrow(spans, "<->") {
        let question_raw = format!("{} <-> ?", render(&lhs).trim());
        let question = build_context_question(context, depth, &question_raw);
        return Some(Card {
            card_type: CardType::Bidirectional,
            fields: vec![question, render(&rhs).trim().to_string()],
        });
    }

    // Try basic (->).  If a heading is present and context_stack is empty
    // (i.e. this is a top-level list item, not already nested), emit an
    // attribute card with a two-line front.
    if let Some((lhs, rhs)) = split_at_arrow(spans, "->") {
        let key = render(&lhs).trim().to_string();
        let value = render(&rhs).trim().to_string();

        let question = if let Some(subject) = heading {
            // Only apply heading when there is no nested list context —
            // nested items already have their own context chain.
            if context.is_empty() {
                format!("{subject}\n{key} \u{2192}?")
            } else {
                let question_raw = format!("{key} -> ?");
                build_context_question(context, depth, &question_raw)
            }
        } else {
            let question_raw = format!("{key} -> ?");
            build_context_question(context, depth, &question_raw)
        };

        return Some(Card {
            card_type: CardType::Basic,
            fields: vec![question, value],
        });
    }

    None
}

/// Prepend ancestor context items as an indented list before `current`.
fn build_context_question(
    context: &[(usize, String)],
    current_depth: usize,
    current: &str,
) -> String {
    let ancestors: Vec<&str> = context
        .iter()
        .filter(|(d, _)| *d < current_depth)
        .map(|(_, t)| t.as_str())
        .collect();

    if ancestors.is_empty() {
        return current.to_string();
    }

    let mut q = String::new();
    for (i, text) in ancestors.iter().enumerate() {
        q.push_str(&"    ".repeat(i));
        q.push_str("- ");
        q.push_str(text);
        q.push('\n');
    }
    q.push_str(&"    ".repeat(ancestors.len()));
    q.push_str("- ");
    q.push_str(current);
    q
}

// ── DocumentParser impl ───────────────────────────────────────────────────────

impl DocumentParser for MarkdownParser {
    type Error = ParseError;

    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error> {
        // Pre-process: expand template blocks first.  Template code fences
        // paired with pipe tables are expanded into raw text that downstream
        // stages then parse for card syntax.
        let after_templates = expand_templates(markdown)?;

        // Pre-process: extract pipe-table blocks before anything else so that
        // cmark never sees them and doesn't produce spurious output.
        let (pipe_table_cards, after_tables) = extract_pipe_table_cards(&after_templates);

        // Pre-process: extract ordered-sequence blocks before passing to the
        // Markdown parser.  The sequence lines (and their labels) are removed
        // from the residual text so that pulldown_cmark never sees them.
        let (sequence_cards, after_sequences) = extract_sequence_cards(&after_tables);

        // Pre-process: extract block cards (`->` or `<->` on its own line)
        // before the Markdown parser.  The question, arrow, and answer lines
        // are removed from the residual text.
        let (block_cards, residual) = extract_block_cards(&after_sequences);

        let mut state = ParseState::new();
        let mut doc = ParsedDocument::default();
        // Collect raw media refs before dedup.
        let mut media_refs: Vec<MediaReference> = Vec::new();
        // Track which source_paths have already been added (for dedup).
        let mut seen_paths: HashSet<String> = HashSet::new();

        let parse_input = if residual.trim().is_empty() {
            // If only sequences were present, use an empty string; we'll handle
            // the EmptyDocument error below.
            String::new()
        } else {
            residual
        };

        for event in Parser::new_ext(&parse_input, Options::empty()) {
            let card = match event {
                Event::Start(Tag::List(_)) => {
                    state.on_start_list();
                    None
                }
                Event::End(Tag::List(_)) => {
                    state.on_end_list();
                    None
                }
                Event::Start(Tag::Heading(_, _, _)) => {
                    state.on_start_heading();
                    None
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    state.on_end_heading();
                    None
                }
                Event::Start(Tag::Item) => {
                    state.on_start_item();
                    None
                }
                Event::End(Tag::Item) => state.on_end_item(),
                Event::Start(Tag::Image(_, u, _)) => {
                    state.on_start_image(u.to_string());
                    None
                }
                Event::End(Tag::Image(_, _, _)) => {
                    if let Some((url, alt)) = state.on_end_image() {
                        // Extract just the filename component for target_name.
                        let target_name = std::path::Path::new(&url)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&url)
                            .to_string();
                        let alt_text = if alt.is_empty() { None } else { Some(alt) };
                        let media_ref = MediaReference::new(url.clone(), target_name, alt_text, 0)?;
                        // Dedup: keep only the first occurrence of each source_path.
                        if seen_paths.insert(url) {
                            media_refs.push(media_ref);
                        }
                    }
                    None
                }
                Event::Start(Tag::Paragraph) => {
                    state.on_start_paragraph();
                    None
                }
                Event::End(Tag::Paragraph) => state.on_end_paragraph(),
                Event::Text(s) => {
                    state.on_text(s.as_ref());
                    None
                }
                Event::Code(s) => {
                    state.on_code_span(s.as_ref());
                    None
                }
                Event::SoftBreak => {
                    state.on_soft_break();
                    None
                }
                Event::HardBreak => {
                    state.on_hard_break();
                    None
                }
                _ => None,
            };
            if let Some(card) = card {
                doc.cards.push(card);
            }
        }

        doc.media = media_refs;

        // Prepend pipe-table cards, then sequence cards, then block cards,
        // then cmark cards.
        // Order: pipe tables → sequences → block cards → inline cards.
        let mut combined: Vec<Card> = Vec::new();
        combined.extend(pipe_table_cards);
        combined.extend(sequence_cards);
        combined.extend(block_cards);
        combined.extend(doc.cards);
        doc.cards = combined;

        if doc.cards.is_empty() {
            Err(ParseError::EmptyDocument)
        } else {
            Ok(doc)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CardType;

    /// An arrow inside a backtick code span must NOT be treated as a delimiter.
    #[test]
    fn test_arrow_in_code_span_not_split() {
        // The `->` is inside a Code span — split_at_arrow skips it entirely.
        let input = "- `->` is an arrow";
        let result = MarkdownParser::new().parse(input);
        assert!(
            result.is_err(),
            "expected no card for arrow-only-in-code-span, got: {result:?}",
        );
    }

    /// Inline text content is preserved correctly across the arrow split.
    #[test]
    fn test_inline_text_preserved() {
        let input = "- hello world -> answer";
        let result = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(result.cards.len(), 1);
        let card = &result.cards[0];
        assert_eq!(card.card_type, CardType::Basic);
        assert_eq!(card.fields[0], "hello world -> ?");
        assert_eq!(card.fields[1], "answer");
    }

    #[test]
    fn test_image_on_front_creates_media_ref() {
        let input = "- ![](image.jpg) -> answer";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert!(
            doc.cards[0].fields[0].contains("<img"),
            "front: {:?}",
            doc.cards[0].fields[0]
        );
        assert_eq!(doc.cards[0].fields[1], "answer");
        assert_eq!(doc.media.len(), 1);
        assert_eq!(doc.media[0].source_path, "image.jpg");
        assert_eq!(doc.media[0].target_name, "image.jpg");
    }

    #[test]
    fn test_image_on_answer_with_alt_and_dir() {
        let input = "- question -> ![Alt](dir/image.png)";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert!(
            doc.cards[0].fields[1].contains("<img"),
            "answer: {:?}",
            doc.cards[0].fields[1]
        );
        assert_eq!(doc.media.len(), 1);
        assert_eq!(doc.media[0].source_path, "dir/image.png");
        assert_eq!(doc.media[0].target_name, "image.png");
        assert_eq!(doc.media[0].alt_text, Some("Alt".to_string()));
    }

    #[test]
    fn test_bidirectional_two_images() {
        let input = "- ![](a.jpg) <-> ![](b.jpg)";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert_eq!(doc.media.len(), 2);
        let paths: Vec<&str> = doc.media.iter().map(|m| m.source_path.as_str()).collect();
        assert!(paths.contains(&"a.jpg"), "paths: {paths:?}");
        assert!(paths.contains(&"b.jpg"), "paths: {paths:?}");
    }

    #[test]
    fn test_dedup_same_image_two_cards() {
        let input = "- ![](shared.jpg) -> first\n- ![](shared.jpg) -> second";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 2);
        assert_eq!(doc.media.len(), 1, "expected dedup, got: {:?}", doc.media);
        assert_eq!(doc.media[0].source_path, "shared.jpg");
    }

    #[test]
    fn test_nested_context_with_image() {
        let input = "- Animals\n  - ![](cat.jpg) -> Cat";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert!(
            doc.cards[0].fields[0].contains("Animals"),
            "front: {:?}",
            doc.cards[0].fields[0]
        );
        assert_eq!(doc.media.len(), 1);
        assert_eq!(doc.media[0].source_path, "cat.jpg");
    }

    #[test]
    fn test_card_with_no_images() {
        let input = "- question -> answer";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert!(
            doc.media.is_empty(),
            "expected empty media, got: {:?}",
            doc.media
        );
    }
}
