//! Parser traits and Markdown parser implementation.
use crate::error::ParseError;
use crate::types::{Card, CardType, MediaReference, ParsedDocument};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::HashSet;

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
}

impl ParseState {
    fn new() -> Self {
        Self {
            list_depth: 0,
            context_stack: Vec::new(),
            current_item: None,
            current_image: None,
            paragraph: None,
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
            .map_or(false, |(d, _)| *d > self.list_depth)
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
        make_card(&item.spans, &self.context_stack, item.depth)
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
        make_card(&spans, &[], 0)
    }

    fn on_text(&mut self, s: &str) {
        if let Some(img) = self.current_image.as_mut() {
            img.alt.push_str(s);
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
}

// ── Card construction ─────────────────────────────────────────────────────────

/// Attempt to build a [`Card`] from the accumulated spans of a list item or
/// paragraph. Tries `<->` before `->` so bidirectional cards are preferred.
fn make_card(spans: &[Span], context: &[(usize, String)], depth: usize) -> Option<Card> {
    for (arrow, card_type) in [("<->", CardType::Bidirectional), ("->", CardType::Basic)] {
        if let Some((lhs, rhs)) = split_at_arrow(spans, arrow) {
            let question_raw = format!("{} {arrow} ?", render(&lhs).trim());
            let question = build_context_question(context, depth, &question_raw);
            return Some(Card {
                card_type,
                fields: vec![question, render(&rhs).trim().to_string()],
            });
        }
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
        let mut state = ParseState::new();
        let mut doc = ParsedDocument::default();
        // Collect raw media refs before dedup.
        let mut media_refs: Vec<MediaReference> = Vec::new();
        // Track which source_paths have already been added (for dedup).
        let mut seen_paths: HashSet<String> = HashSet::new();

        for event in Parser::new_ext(markdown, Options::empty()) {
            let card = match event {
                Event::Start(Tag::List(_)) => {
                    state.on_start_list();
                    None
                }
                Event::End(Tag::List(_)) => {
                    state.on_end_list();
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
