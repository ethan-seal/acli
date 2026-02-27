//! Parser traits and Markdown parser implementation.
use crate::error::ParseError;
use crate::types::{Card, CardType, ParsedDocument};
use pulldown_cmark::{Event, Options, Parser, Tag};

/// Trait for parsing documents into cards.
pub trait DocumentParser {
    /// Error type returned by the parser.
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Parse a Markdown string into a parsed document containing cards.
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error>;
}

/// Parser that recognizes simple `->` and `<->` card syntax in Markdown.
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

impl DocumentParser for MarkdownParser {
    type Error = ParseError;

    /// Walk pulldown-cmark events with a state machine to extract cards.
    ///
    /// State:
    /// - `list_depth`     – increments on `Start(List)`, decrements on `End(List)`
    /// - `item_depth`     – increments on `Start(Item)`, decrements on `End(Item)`
    /// - `context_stack`  – (item_depth, text) pairs for ancestor context items
    /// - `item_text`      – accumulated rendered text for the current list item
    /// - `item_has_sublist` – true when the current item contains a nested list
    /// - `in_image`       – true while collecting alt text for an `![alt](url)` image
    /// - `in_paragraph`   – true inside a top-level paragraph (non-list)
    ///
    /// Inline code spans (`Event::Code`) are wrapped in `\x00CODE\x00…\x00ENDCODE\x00`
    /// sentinels so that arrows inside them are not treated as delimiters.
    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error> {
        let mut doc = ParsedDocument::default();

        let parser = Parser::new_ext(markdown, Options::empty());

        let mut list_depth: usize = 0;
        let mut item_depth: usize = 0;
        // (depth_when_pushed, text) — ancestors used to build context questions
        let mut context_stack: Vec<(usize, String)> = Vec::new();

        // Per-item state
        let mut item_text = String::new();
        let mut item_has_sublist = false;
        let mut current_item_depth: usize = 0;

        // Image accumulation
        let mut in_image = false;
        let mut image_url = String::new();
        let mut image_alt = String::new();

        // Paragraph (non-list) text accumulation
        let mut in_paragraph = false;
        let mut paragraph_text = String::new();

        for event in parser {
            match event {
                // ── List boundaries ───────────────────────────────────────────
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                    if item_depth > 0 {
                        // A nested list started inside this item → it becomes a
                        // context parent.  Snapshot its text onto the stack.
                        item_has_sublist = true;
                        let text = item_text.trim().to_string();
                        context_stack.push((current_item_depth, text));
                        item_text.clear();
                    }
                }

                Event::End(Tag::List(_)) => {
                    list_depth -= 1;
                    // Pop context entries that belong to the list level we just left.
                    while context_stack.last().map_or(false, |(d, _)| *d > list_depth) {
                        context_stack.pop();
                    }
                }

                // ── Item boundaries ───────────────────────────────────────────
                Event::Start(Tag::Item) => {
                    item_depth += 1;
                    current_item_depth = item_depth;
                    item_text.clear();
                    item_has_sublist = false;
                }

                Event::End(Tag::Item) => {
                    // Leaf items (no sub-list) are candidates for cards.
                    if !item_has_sublist {
                        let text = item_text.trim().to_string();
                        if let Some(card) = make_card(&text, &context_stack, current_item_depth) {
                            doc.cards.push(card);
                        }
                    }
                    item_depth -= 1;
                    item_text.clear();
                    item_has_sublist = false;
                }

                // ── Images ────────────────────────────────────────────────────
                Event::Start(Tag::Image(_, url, _)) => {
                    in_image = true;
                    image_url = url.to_string();
                    image_alt.clear();
                }

                Event::End(Tag::Image(_, _, _)) => {
                    let img = format!("<img src=\"{}\" alt=\"{}\">", image_url, image_alt);
                    if item_depth > 0 {
                        item_text.push_str(&img);
                    } else if in_paragraph {
                        paragraph_text.push_str(&img);
                    }
                    in_image = false;
                    image_url.clear();
                    image_alt.clear();
                }

                // ── Paragraphs (non-list content like "One -> 1") ─────────────
                Event::Start(Tag::Paragraph) if list_depth == 0 => {
                    in_paragraph = true;
                    paragraph_text.clear();
                }

                Event::End(Tag::Paragraph) if list_depth == 0 => {
                    in_paragraph = false;
                    let text = paragraph_text.trim().to_string();
                    if let Some(card) = make_card(&text, &[], 0) {
                        doc.cards.push(card);
                    }
                    paragraph_text.clear();
                }

                // ── Inline code spans ─────────────────────────────────────────
                // pulldown-cmark emits `Event::Code(s)` as a single leaf event
                // (no Start/End pair).  We wrap the content in sentinel bytes so
                // the arrow scanner knows to skip it.
                Event::Code(s) => {
                    let marker = format!("\x00CODE\x00{}\x00ENDCODE\x00", s.as_ref());
                    if item_depth > 0 {
                        item_text.push_str(&marker);
                    } else if in_paragraph {
                        paragraph_text.push_str(&marker);
                    }
                }

                // ── Text / whitespace ─────────────────────────────────────────
                Event::Text(s) => {
                    let text = s.as_ref();
                    if in_image {
                        image_alt.push_str(text);
                    } else if item_depth > 0 {
                        item_text.push_str(text);
                    } else if in_paragraph {
                        paragraph_text.push_str(text);
                    }
                }

                Event::SoftBreak => {
                    if item_depth > 0 {
                        item_text.push(' ');
                    } else if in_paragraph {
                        paragraph_text.push(' ');
                    }
                }

                Event::HardBreak => {
                    if item_depth > 0 {
                        item_text.push('\n');
                    } else if in_paragraph {
                        paragraph_text.push('\n');
                    }
                }

                _ => {}
            }
        }

        if doc.cards.is_empty() {
            Err(ParseError::EmptyDocument)
        } else {
            Ok(doc)
        }
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

/// Attempt to build a [`Card`] from the accumulated text of a list item or paragraph.
///
/// Arrow detection (`<->` / `->`) skips over inline-code sentinels so that
/// `` `->` `` is never treated as a delimiter.
fn make_card(text: &str, context_stack: &[(usize, String)], current_depth: usize) -> Option<Card> {
    if let Some(pos) = find_arrow_outside_code(text, "<->") {
        let lhs = restore_code(&text[..pos]).trim().to_string();
        let rhs = restore_code(&text[pos + "<->".len()..]).trim().to_string();
        let question_raw = format!("{} <-> ?", lhs);
        let question = build_context_question(context_stack, current_depth, &question_raw);
        Some(Card {
            card_type: CardType::Bidirectional,
            fields: vec![question, rhs],
        })
    } else if let Some(pos) = find_arrow_outside_code(text, "->") {
        let lhs = restore_code(&text[..pos]).trim().to_string();
        let rhs = restore_code(&text[pos + "->".len()..]).trim().to_string();
        let question_raw = format!("{} -> ?", lhs);
        let question = build_context_question(context_stack, current_depth, &question_raw);
        Some(Card {
            card_type: CardType::Basic,
            fields: vec![question, rhs],
        })
    } else {
        None
    }
}

/// Find the byte position of `needle` in `haystack`, skipping over any
/// `\x00CODE\x00…\x00ENDCODE\x00` spans (inline code sentinels).
fn find_arrow_outside_code(haystack: &str, needle: &str) -> Option<usize> {
    const CODE_START: &str = "\x00CODE\x00";
    const CODE_END: &str = "\x00ENDCODE\x00";

    let mut i = 0;
    while i < haystack.len() {
        if haystack[i..].starts_with(CODE_START) {
            // Jump past the entire code sentinel span.
            let after = i + CODE_START.len();
            if let Some(end_off) = haystack[after..].find(CODE_END) {
                i = after + end_off + CODE_END.len();
            } else {
                break; // Malformed sentinel; stop scanning.
            }
            continue;
        }
        if haystack[i..].starts_with(needle) {
            return Some(i);
        }
        // Advance by one UTF-8 character.
        i += haystack[i..].chars().next().map_or(1, |c| c.len_utf8());
    }
    None
}

/// Remove `\x00CODE\x00…\x00ENDCODE\x00` sentinels, keeping the inner content.
fn restore_code(text: &str) -> String {
    const CODE_START: &str = "\x00CODE\x00";
    const CODE_END: &str = "\x00ENDCODE\x00";
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(s) = rest.find(CODE_START) {
        result.push_str(&rest[..s]);
        let after = &rest[s + CODE_START.len()..];
        if let Some(e) = after.find(CODE_END) {
            result.push_str(&after[..e]);
            rest = &after[e + CODE_END.len()..];
        } else {
            result.push_str(after);
            return result;
        }
    }
    result.push_str(rest);
    result
}

/// Build the question field with ancestor context prepended as a nested list.
fn build_context_question(
    context_stack: &[(usize, String)],
    current_depth: usize,
    current: &str,
) -> String {
    // Ancestors are entries with depth strictly less than the current item's depth.
    let ancestors: Vec<&str> = context_stack
        .iter()
        .filter(|(d, _)| *d < current_depth)
        .map(|(_, t)| t.as_str())
        .collect();

    if ancestors.is_empty() {
        return current.to_string();
    }

    let mut q = String::new();
    for (i, text) in ancestors.iter().enumerate() {
        let indent = "    ".repeat(i);
        q.push_str(&indent);
        q.push_str("- ");
        q.push_str(text);
        q.push('\n');
    }
    let indent = "    ".repeat(ancestors.len());
    q.push_str(&indent);
    q.push_str("- ");
    q.push_str(current);
    q
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CardType;

    /// An arrow inside a backtick code span must NOT be treated as a delimiter.
    #[test]
    fn test_arrow_in_code_span_not_split() {
        // "`->`" is entirely inside a code span — no card should be produced.
        let input = "- `->` is an arrow";
        let parser = MarkdownParser::new();
        let result = parser.parse(input);
        assert!(
            result.is_err(),
            "Expected no card for arrow-only-in-code-span, got: {:?}",
            result
        );
    }

    /// Inline text content is preserved correctly across the arrow split.
    #[test]
    fn test_inline_text_preserved() {
        let input = "- hello world -> answer";
        let parser = MarkdownParser::new();
        let result = parser.parse(input).unwrap();
        assert_eq!(result.cards.len(), 1);
        assert_eq!(result.cards[0].card_type, CardType::Basic);
        assert_eq!(result.cards[0].fields[0], "hello world -> ?");
        assert_eq!(result.cards[0].fields[1], "answer");
    }
}
