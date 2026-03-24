//! Parser traits and Markdown parser implementation.

mod block;
mod extraction;
mod inline;
mod pipe_table;
mod sequence;
mod template;

use crate::error::ParseError;
use crate::types::{Card, MediaReference, ParsedDocument};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::HashSet;

use block::extract_block_cards;
use inline::ParseState;
use pipe_table::extract_pipe_table_cards;
use sequence::extract_sequence_cards;
use template::expand_templates;

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

impl DocumentParser for MarkdownParser {
    type Error = ParseError;

    fn parse(&self, markdown: &str) -> Result<ParsedDocument, Self::Error> {
        // Pre-process: expand template blocks first.
        let (after_templates, template_warnings) = expand_templates(markdown)?;

        // Pre-process: extract pipe-table blocks.
        let pipe_result = extract_pipe_table_cards(&after_templates);

        // Pre-process: extract ordered-sequence blocks.
        let sequence_result = extract_sequence_cards(&pipe_result.residual);

        // Pre-process: extract block cards.
        let block_result = extract_block_cards(&sequence_result.residual);

        let mut state = ParseState::new();
        let mut doc = ParsedDocument::default();
        let mut media_refs: Vec<MediaReference> = Vec::new();
        let mut seen_paths: HashSet<String> = HashSet::new();

        let parse_input = if block_result.residual.trim().is_empty() {
            String::new()
        } else {
            block_result.residual
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
                        let is_remote = url.starts_with("http://") || url.starts_with("https://");
                        if !is_remote {
                            let target_name = std::path::Path::new(&url)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&url)
                                .to_string();
                            let alt_text = if alt.is_empty() { None } else { Some(alt) };
                            let media_ref =
                                MediaReference::new(url.clone(), target_name, alt_text, 0)?;
                            if seen_paths.insert(url) {
                                media_refs.push(media_ref);
                            }
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
        doc.warnings = template_warnings;
        doc.warnings.extend(block_result.warnings);

        // Combine cards in order: pipe tables → sequences → block cards → inline cards.
        let mut combined: Vec<Card> = Vec::new();
        combined.extend(pipe_result.cards);
        combined.extend(sequence_result.cards);
        combined.extend(block_result.cards);
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

    #[test]
    fn test_arrow_in_code_span_not_split() {
        let input = "- `->` is an arrow";
        let result = MarkdownParser::new().parse(input);
        assert!(
            result.is_err(),
            "expected no card for arrow-only-in-code-span, got: {result:?}",
        );
    }

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

    #[test]
    fn test_remote_image_url_not_treated_as_media() {
        let input = "- ![badge](https://example.com/badge.svg?branch=main) -> answer";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert!(
            doc.cards[0].fields[0].contains("<img"),
            "URL image should still render as <img> in HTML"
        );
        assert!(
            doc.media.is_empty(),
            "remote URLs should not be collected as media: {:?}",
            doc.media
        );
    }

    #[test]
    fn test_http_image_skipped_but_local_image_collected() {
        let input = "- ![](https://example.com/remote.png) ![](local.jpg) -> answer";
        let doc = MarkdownParser::new().parse(input).unwrap();
        assert_eq!(doc.cards.len(), 1);
        assert_eq!(doc.media.len(), 1, "only local image should be media");
        assert_eq!(doc.media[0].source_path, "local.jpg");
    }
}
