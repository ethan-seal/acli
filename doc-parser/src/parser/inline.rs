use crate::types::{Card, CardType};

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
            Span::Image { url, alt } => {
                out.push_str(&format!(r#"<img src="{url}" alt="{alt}">"#))
            }
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
pub(super) struct ParseState {
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
    pub(super) fn new() -> Self {
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
    fn text_sink(&mut self) -> Option<&mut Vec<Span>> {
        if self.current_image.is_some() {
            return None;
        }
        if let Some(item) = self.current_item.as_mut() {
            return Some(&mut item.spans);
        }
        self.paragraph.as_mut()
    }

    /// Return the span buffer for break events (soft/hard).
    fn break_sink(&mut self) -> Option<&mut Vec<Span>> {
        if let Some(item) = self.current_item.as_mut() {
            return Some(&mut item.spans);
        }
        self.paragraph.as_mut()
    }

    // ── Event handlers ────────────────────────────────────────────────────────

    pub(super) fn on_start_list(&mut self) {
        self.list_depth += 1;
        if let Some(item) = self.current_item.as_mut() {
            item.has_sublist = true;
            let text = render(&item.spans).trim().to_string();
            self.context_stack.push((item.depth, text));
            item.spans.clear();
        }
    }

    pub(super) fn on_end_list(&mut self) {
        self.list_depth -= 1;
        while self
            .context_stack
            .last()
            .is_some_and(|(d, _)| *d > self.list_depth)
        {
            self.context_stack.pop();
        }
    }

    pub(super) fn on_start_item(&mut self) {
        self.current_item = Some(ItemAccum::new(self.list_depth));
    }

    /// Consumes the current item and returns a `Card` if it is a leaf item
    /// with an arrow delimiter; returns `None` for context parents or items
    /// without a recognisable delimiter.
    pub(super) fn on_end_item(&mut self) -> Option<Card> {
        let item = self.current_item.take()?;
        if item.has_sublist {
            return None;
        }
        make_card(
            &item.spans,
            &self.context_stack,
            item.depth,
            self.current_heading.as_deref(),
        )
    }

    pub(super) fn on_start_image(&mut self, url: String) {
        self.current_image = Some(ImageAccum {
            url,
            alt: String::new(),
        });
    }

    /// Finalises the current image accumulator, pushing an `Image` span into
    /// the active buffer and returning `(url, alt)` so the caller can create a
    /// `MediaReference`.
    pub(super) fn on_end_image(&mut self) -> Option<(String, String)> {
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

    pub(super) fn on_start_paragraph(&mut self) {
        if self.list_depth == 0 {
            self.paragraph = Some(Vec::new());
        }
    }

    pub(super) fn on_end_paragraph(&mut self) -> Option<Card> {
        let spans = self.paragraph.take()?;
        make_card(&spans, &[], 0, None)
    }

    pub(super) fn on_text(&mut self, s: &str) {
        if let Some(img) = self.current_image.as_mut() {
            img.alt.push_str(s);
        } else if let Some(h) = self.heading_accum.as_mut() {
            h.push_str(s);
        } else if let Some(buf) = self.text_sink() {
            push_text(buf, s);
        }
    }

    pub(super) fn on_code_span(&mut self, s: &str) {
        if let Some(buf) = self.break_sink() {
            buf.push(Span::Code(s.to_string()));
        }
    }

    pub(super) fn on_soft_break(&mut self) {
        if let Some(buf) = self.break_sink() {
            push_text(buf, " ");
        }
    }

    pub(super) fn on_hard_break(&mut self) {
        if let Some(buf) = self.break_sink() {
            push_text(buf, "\n");
        }
    }

    pub(super) fn on_start_heading(&mut self) {
        self.heading_accum = Some(String::new());
    }

    pub(super) fn on_end_heading(&mut self) {
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
            if context.is_empty() {
                format!("{subject}\n{key} -> ?")
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
