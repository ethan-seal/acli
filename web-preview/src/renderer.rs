//! HTML page rendering for card preview.

use crate::PreviewData;
use std::fmt::Write;

/// Allocate an HTML string pre-populated with the `<head>` and CSS.
/// The caller appends `<body>…</body></html>`.
fn page_shell(capacity: usize) -> String {
    let mut html = String::with_capacity(capacity);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>acli - Card Preview</title>\n");
    html.push_str("<style>\n");
    html.push_str(CSS);
    html.push_str("\n</style>\n</head>\n");
    html
}

/// Render the main preview page with cards.
pub(crate) fn render_page(data: &PreviewData) -> String {
    let mut html = page_shell(16384);

    // ── Body ─────────────────────────────────────────────────────────────
    html.push_str("<body>\n<div class=\"container\">\n");

    // Header
    let card_count = data.cards.len();
    let file_count = data.files_processed;
    let _ = write!(
        html,
        "<header>\
           <h1>Card Preview</h1>\
           <div class=\"meta\">\
             <span>Deck: <strong>{}</strong></span>\
             <span>{card_count} card{} from {file_count} file{}</span>\
           </div>\
         </header>\n",
        escape_html(&data.deck_name),
        if card_count == 1 { "" } else { "s" },
        if file_count == 1 { "" } else { "s" },
    );

    // Errors / warnings
    if !data.errors.is_empty() {
        html.push_str("<div class=\"errors-section\">\n<h2>Warnings</h2>\n");
        for e in &data.errors {
            let _ = write!(html, "<div class=\"error-item\">{}</div>\n", escape_html(e));
        }
        html.push_str("</div>\n");
    }

    // Cards (grouped by source file)
    if data.cards.is_empty() {
        html.push_str(
            "<div class=\"empty-state\">\
               <h2>No cards found</h2>\
               <p>Add flashcard syntax to your markdown files and refresh.</p>\
             </div>\n",
        );
    } else {
        push_card_sections(&mut html, data);
    }

    // Footer
    html.push_str("<footer>Served by acli &middot; Refresh the page to pick up changes</footer>\n");
    html.push_str("</div>\n</body>\n</html>");

    html
}

/// Group cards by source file and emit a `<section>` with a table per file.
fn push_card_sections(html: &mut String, data: &PreviewData) {
    // Group consecutive cards by source file.
    let mut groups: Vec<(&str, Vec<(usize, &crate::PreviewCard)>)> = Vec::new();

    for (i, card) in data.cards.iter().enumerate() {
        let file = card.source_file.as_deref().unwrap_or("Unknown source");
        match groups.last_mut() {
            Some((current_file, cards)) if *current_file == file => {
                cards.push((i + 1, card));
            }
            _ => {
                groups.push((file, vec![(i + 1, card)]));
            }
        }
    }

    for (file, cards) in &groups {
        let count = cards.len();
        let _ = write!(
            html,
            "<div class=\"file-section\">\
               <div class=\"file-header\">\
                 <span class=\"file-path\">{}</span>\
                 <span class=\"file-count\">{count} card{}</span>\
               </div>\n",
            escape_html(file),
            if count == 1 { "" } else { "s" },
        );

        html.push_str(
            "<table class=\"card-table\">\
               <thead><tr>\
                 <th>#</th><th>Front</th><th>Back</th><th>Type</th>\
               </tr></thead>\n<tbody>\n",
        );

        for (idx, card) in cards {
            let front_html = text_to_html(&card.front);
            let back_html = text_to_html(&card.back);
            let _ = write!(
                html,
                "<tr class=\"card-row\">\
                   <td class=\"card-index\">{idx}</td>\
                   <td class=\"card-front\"><div class=\"card-content\">{front_html}</div></td>\
                   <td class=\"card-back\"><div class=\"card-content\">{back_html}</div></td>\
                   <td><span class=\"type-badge type-{}\">{}</span></td>\
                 </tr>\n",
                card.card_type.css_class(),
                card.card_type.label(),
            );
        }

        html.push_str("</tbody></table>\n</div>\n");
    }
}

/// Render a minimal error page when the refresh closure fails entirely.
pub(crate) fn render_error_page(message: &str) -> String {
    let mut html = page_shell(2048);
    html.push_str("<body>\n<div class=\"container\">\n");
    html.push_str("<header><h1>Card Preview</h1></header>\n");
    let _ = write!(
        html,
        "<div class=\"errors-section\">\
           <h2>Error</h2>\
           <div class=\"error-item\">{}</div>\
         </div>\n",
        escape_html(message),
    );
    html.push_str("<footer>Served by acli &middot; Fix the issue and refresh</footer>\n");
    html.push_str("</div>\n</body>\n</html>");
    html
}

/// Convert card field text to HTML via pulldown-cmark.
///
/// Single newlines become hard breaks (matching acli's Anki rendering).
fn text_to_html(text: &str) -> String {
    let with_hard_breaks = text.replace('\n', "  \n");
    let parser = pulldown_cmark::Parser::new(&with_hard_breaks);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html.trim_end().to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Self-contained CSS for the preview page.
///
/// Dark theme borrowed from the e2e demo report with card-table styling.
const CSS: &str = r#"
:root {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --bg-card: #0f3460;
  --text-primary: #eaeaea;
  --text-secondary: #b8b8b8;
  --accent: #e94560;
  --success: #00d26a;
  --error: #ff4757;
  --border: #2a2a4a;
}
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
  line-height: 1.6;
  padding: 2rem;
}
.container { max-width: 1200px; margin: 0 auto; }

/* Header */
header {
  text-align: center;
  margin-bottom: 2rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid var(--border);
}
header h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  background: linear-gradient(135deg, var(--accent), #ff6b6b);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.meta {
  color: var(--text-secondary);
  font-size: 0.9rem;
  display: flex;
  justify-content: center;
  gap: 2rem;
}

/* File sections */
.file-section {
  background: var(--bg-secondary);
  border-radius: 12px;
  margin-bottom: 1.5rem;
  overflow: hidden;
  border: 1px solid var(--border);
}
.file-header {
  background: var(--bg-card);
  padding: 0.75rem 1.25rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.file-path {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85rem;
  color: var(--text-secondary);
}
.file-count {
  color: var(--text-secondary);
  font-size: 0.8rem;
}

/* Card table */
.card-table { width: 100%; border-collapse: collapse; }
.card-table th {
  padding: 0.6rem 1rem;
  text-align: left;
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.7rem;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border);
}
.card-table td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border);
  vertical-align: top;
}
.card-table tr:last-child td { border-bottom: none; }
.card-row:hover { background: rgba(255,255,255,0.03); }
.card-index { color: var(--text-secondary); font-size: 0.8rem; width: 40px; }
.card-content { font-size: 0.9rem; line-height: 1.5; }
.card-content ul { margin: 0; padding-left: 1.5rem; }
.card-content li { margin: 0.15rem 0; }
.card-content p { margin: 0; }
.card-content img { max-width: 200px; border-radius: 4px; }
.card-front { width: 35%; }
.card-back { width: 35%; }

/* Type badges */
.type-badge {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  white-space: nowrap;
}
.type-basic    { background: rgba(138,180,248,0.2); color: #8ab4f8; }
.type-bidi     { background: rgba(138,138,255,0.2); color: #8a8aff; }
.type-sequence { background: rgba(255,193,7,0.2);   color: #ffc107; }

/* Errors */
.errors-section {
  background: rgba(255,71,87,0.1);
  border: 1px solid var(--error);
  border-radius: 12px;
  padding: 1.25rem;
  margin-bottom: 1.5rem;
}
.errors-section h2 { color: var(--error); font-size: 1rem; margin-bottom: 0.75rem; }
.error-item {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85rem;
  color: var(--error);
  padding: 0.25rem 0;
}

/* Empty state */
.empty-state { text-align: center; padding: 3rem; color: var(--text-secondary); }
.empty-state h2 { margin-bottom: 0.5rem; }

/* Footer */
footer {
  text-align: center;
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 0.8rem;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CardType, PreviewCard, PreviewData};

    fn sample_data() -> PreviewData {
        PreviewData {
            deck_name: "Test::Deck".into(),
            files_processed: 2,
            errors: vec![],
            cards: vec![
                PreviewCard {
                    front: "hello".into(),
                    back: "world".into(),
                    card_type: CardType::Basic,
                    source_file: Some("chapter1.md".into()),
                },
                PreviewCard {
                    front: "hola".into(),
                    back: "hello".into(),
                    card_type: CardType::Bidirectional,
                    source_file: Some("chapter1.md".into()),
                },
                PreviewCard {
                    front: "Boot sequence\n1/2: power on".into(),
                    back: "press button".into(),
                    card_type: CardType::Sequence,
                    source_file: Some("chapter2.md".into()),
                },
            ],
        }
    }

    #[test]
    fn render_page_contains_deck_name() {
        let html = render_page(&sample_data());
        assert!(html.contains("Test::Deck"));
    }

    #[test]
    fn render_page_contains_card_content() {
        let html = render_page(&sample_data());
        assert!(html.contains("hello"));
        assert!(html.contains("world"));
        assert!(html.contains("hola"));
    }

    #[test]
    fn render_page_groups_by_file() {
        let html = render_page(&sample_data());
        assert!(html.contains("chapter1.md"));
        assert!(html.contains("chapter2.md"));
        // chapter1 has 2 cards
        assert!(html.contains("2 cards"));
        // chapter2 has 1 card
        assert!(html.contains("1 card"));
    }

    #[test]
    fn render_page_contains_type_badges() {
        let html = render_page(&sample_data());
        assert!(html.contains("type-basic"));
        assert!(html.contains("type-bidi"));
        assert!(html.contains("type-sequence"));
    }

    #[test]
    fn render_page_renders_markdown_to_html() {
        let data = PreviewData {
            deck_name: "Test".into(),
            files_processed: 1,
            errors: vec![],
            cards: vec![PreviewCard {
                front: "**bold** text".into(),
                back: "- item 1\n- item 2".into(),
                card_type: CardType::Basic,
                source_file: Some("test.md".into()),
            }],
        };
        let html = render_page(&data);
        assert!(
            html.contains("<strong>bold</strong>"),
            "should render bold: {html}"
        );
        assert!(html.contains("<li>"), "should render list items: {html}");
    }

    #[test]
    fn render_page_shows_empty_state() {
        let data = PreviewData {
            deck_name: "Empty".into(),
            files_processed: 3,
            errors: vec![],
            cards: vec![],
        };
        let html = render_page(&data);
        assert!(html.contains("No cards found"));
        assert!(html.contains("0 cards"));
    }

    #[test]
    fn render_page_shows_errors() {
        let data = PreviewData {
            deck_name: "Test".into(),
            files_processed: 1,
            errors: vec!["bad.md: unexpected token".into()],
            cards: vec![],
        };
        let html = render_page(&data);
        assert!(html.contains("Warnings"));
        assert!(html.contains("bad.md: unexpected token"));
    }

    #[test]
    fn render_page_escapes_html_in_deck_name() {
        let data = PreviewData {
            deck_name: "Test<script>alert(1)</script>".into(),
            files_processed: 0,
            errors: vec![],
            cards: vec![],
        };
        let html = render_page(&data);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn render_error_page_contains_message() {
        let html = render_error_page("something broke");
        assert!(html.contains("something broke"));
        assert!(html.contains("Error"));
    }

    #[test]
    fn text_to_html_renders_basic_text() {
        let result = text_to_html("hello world");
        assert_eq!(result, "<p>hello world</p>");
    }

    #[test]
    fn text_to_html_converts_newlines_to_breaks() {
        let result = text_to_html("line1\nline2");
        assert!(
            result.contains("<br />"),
            "should have line break: {result}"
        );
    }
}
