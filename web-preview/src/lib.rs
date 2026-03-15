//! Lightweight web preview server for acli flashcards.
//!
//! Serves a self-contained HTML page showing parsed cards grouped by source file.
//! The `refresh` closure is called on every page load so changes are picked up
//! with a simple browser refresh — no file watcher needed.

use std::fmt::Write;
use std::path::{Path, PathBuf};

// ── Public types ─────────────────────────────────────────────────────────────

/// Card type for preview display.
#[derive(Debug, Clone)]
pub enum CardType {
    Basic,
    Bidirectional,
    Sequence,
}

impl CardType {
    fn label(&self) -> &'static str {
        match self {
            CardType::Basic => "Basic",
            CardType::Bidirectional => "Bidirectional",
            CardType::Sequence => "Sequence",
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            CardType::Basic => "basic",
            CardType::Bidirectional => "bidi",
            CardType::Sequence => "sequence",
        }
    }
}

/// A card to display in the web preview.
#[derive(Debug, Clone)]
pub struct PreviewCard {
    /// Raw text for the front of the card (will be rendered as markdown).
    pub front: String,
    /// Raw text for the back of the card (will be rendered as markdown).
    pub back: String,
    /// Card type.
    pub card_type: CardType,
    /// Source file path (for grouping).
    pub source_file: Option<String>,
}

/// Data for rendering a preview page.
#[derive(Debug, Clone)]
pub struct PreviewData {
    /// Deck name for display.
    pub deck_name: String,
    /// All cards to display.
    pub cards: Vec<PreviewCard>,
    /// Number of markdown files scanned.
    pub files_processed: usize,
    /// Non-fatal warnings or per-file parse errors.
    pub errors: Vec<String>,
}

// ── Server ───────────────────────────────────────────────────────────────────

/// Start a blocking web server that serves the card preview page.
///
/// The `refresh` closure is called on **every** page load, so edits to the
/// underlying markdown files are reflected immediately on browser refresh.
///
/// When `static_dir` is `Some`, requests under `/media/…` are served as static
/// files from that directory.  Card fields should use [`rewrite_image_urls`] so
/// that `<img src="…">` paths resolve correctly against this route.
pub fn serve<F>(
    port: u16,
    static_dir: Option<PathBuf>,
    refresh: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Result<PreviewData, String>,
{
    let addr = format!("0.0.0.0:{port}");
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| format!("failed to bind to port {port}: {e}"))?;

    eprintln!("Serving card preview at http://localhost:{port}");
    eprintln!("Press Ctrl+C to stop.");

    loop {
        let request = match server.recv() {
            Ok(req) => req,
            Err(_) => break,
        };

        // Own the URL so `request` is free for `respond()`.
        let url = request.url().to_string();

        if url == "/" {
            let (html, status) = match refresh() {
                Ok(data) => (render_page(&data), 200),
                Err(e) => (render_error_page(&e), 500),
            };
            let response = tiny_http::Response::from_string(&html)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"text/html; charset=utf-8"[..],
                    )
                    .unwrap(),
                );
            let _ = request.respond(response);
        } else if url == "/favicon.ico" {
            let response = tiny_http::Response::from_string("").with_status_code(204);
            let _ = request.respond(response);
        } else if let Some(rel_path) = url.strip_prefix("/media/") {
            serve_static_file(request, &static_dir, rel_path);
        } else {
            let response = tiny_http::Response::from_string("Not Found").with_status_code(404);
            let _ = request.respond(response);
        }
    }

    Ok(())
}

// ── Static file serving ──────────────────────────────────────────────────────

/// Serve a static file from `static_dir` at the given relative path.
fn serve_static_file(request: tiny_http::Request, static_dir: &Option<PathBuf>, rel_path: &str) {
    let respond_404 = |req: tiny_http::Request| {
        let _ = req.respond(tiny_http::Response::from_string("Not Found").with_status_code(404));
    };

    let Some(root) = static_dir.as_ref() else {
        respond_404(request);
        return;
    };

    let decoded = percent_decode(rel_path);
    let file_path = root.join(&decoded);

    // Prevent directory traversal: canonical path must stay inside root.
    let Ok(canonical) = file_path.canonicalize() else {
        respond_404(request);
        return;
    };
    let Ok(root_canonical) = root.canonicalize() else {
        respond_404(request);
        return;
    };
    if !canonical.starts_with(&root_canonical) || !canonical.is_file() {
        respond_404(request);
        return;
    }

    let file = match std::fs::File::open(&canonical) {
        Ok(f) => f,
        Err(_) => {
            respond_404(request);
            return;
        }
    };

    let content_type = mime_for_path(&canonical);
    let header =
        tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();
    let response = tiny_http::Response::from_file(file).with_header(header);
    let _ = request.respond(response);
}

/// Map a file extension to a MIME type (covers common image formats).
fn mime_for_path(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

/// Decode percent-encoded URL characters (`%20` → space, etc.).
fn percent_decode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut bytes = input.as_bytes().iter().copied();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next().and_then(hex_val);
            let lo = bytes.next().and_then(hex_val);
            match (hi, lo) {
                (Some(h), Some(l)) => output.push((h << 4 | l) as char),
                _ => output.push('%'), // malformed sequence — keep literal
            }
        } else {
            output.push(b as char);
        }
    }
    output
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ── Image URL rewriting ──────────────────────────────────────────────────────

/// Rewrite relative `<img src="…">` URLs in card text to use the `/media/`
/// route so the preview server can serve them.
///
/// `source_dir` is the directory of the markdown file **relative to the
/// server's static root** (i.e. relative to CWD).  Remote URLs (`http(s)://`),
/// data URIs, and already-absolute paths are left unchanged.
pub fn rewrite_image_urls(text: &str, source_dir: &Path) -> String {
    // Two patterns to handle:
    //  1. <img src="url">   — inline cards (doc-parser pre-renders these)
    //  2. ![alt](url)       — block cards (raw markdown kept as-is)
    let after_html = rewrite_html_image_srcs(text, source_dir);
    rewrite_markdown_images(&after_html, source_dir)
}

/// Rewrite `<img src="url" …>` HTML tags.
fn rewrite_html_image_srcs(text: &str, source_dir: &Path) -> String {
    const NEEDLE: &str = "<img src=\"";
    let mut result = String::with_capacity(text.len() + 64);
    let mut remaining = text;

    while let Some(idx) = remaining.find(NEEDLE) {
        result.push_str(&remaining[..idx + NEEDLE.len()]);
        remaining = &remaining[idx + NEEDLE.len()..];

        if let Some(end) = remaining.find('"') {
            let url = &remaining[..end];
            push_rewritten_url(&mut result, url, source_dir);
            remaining = &remaining[end..];
        }
    }

    result.push_str(remaining);
    result
}

/// Rewrite `![alt](url)` markdown image references.
fn rewrite_markdown_images(text: &str, source_dir: &Path) -> String {
    let mut result = String::with_capacity(text.len() + 64);
    let mut remaining = text;

    while let Some(idx) = remaining.find("![") {
        // Copy everything before ![
        result.push_str(&remaining[..idx]);
        remaining = &remaining[idx..];

        // Find the ]( that closes the alt-text bracket.
        if let Some(bracket_paren) = remaining.find("](") {
            let url_start = bracket_paren + "]((".len() - 1; // index of '(' + 1
            let after_paren = &remaining[url_start..];

            if let Some(close) = after_paren.find(')') {
                let url = &after_paren[..close];
                // Copy ![alt](
                result.push_str(&remaining[..url_start]);
                push_rewritten_url(&mut result, url, source_dir);
                // Skip past the closing )
                remaining = &after_paren[close..];
            } else {
                // No closing ) — not a valid image, copy ![ and continue
                result.push_str("![");
                remaining = &remaining["![".len()..];
            }
        } else {
            // No ]( — not a valid image, copy ![ and continue
            result.push_str("![");
            remaining = &remaining["![".len()..];
        }
    }

    result.push_str(remaining);
    result
}

/// Append `url` to `result`, rewriting relative paths to use `/media/`.
fn push_rewritten_url(result: &mut String, url: &str, source_dir: &Path) {
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with('/')
        || url.starts_with("data:")
    {
        result.push_str(url);
    } else {
        let resolved = source_dir.join(url);
        result.push_str("/media/");
        result.push_str(&resolved.to_string_lossy().replace('\\', "/"));
    }
}

// ── Rendering helpers ────────────────────────────────────────────────────────

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

// ── Page rendering ───────────────────────────────────────────────────────────

fn render_page(data: &PreviewData) -> String {
    let mut html = String::with_capacity(16384);

    // ── Head + CSS ───────────────────────────────────────────────────────
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>acli - Card Preview</title>\n");
    html.push_str("<style>\n");
    html.push_str(CSS);
    html.push_str("\n</style>\n</head>\n");

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
    let mut groups: Vec<(&str, Vec<(usize, &PreviewCard)>)> = Vec::new();

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
fn render_error_page(message: &str) -> String {
    let mut html = String::with_capacity(2048);
    html.push_str("<!DOCTYPE html>\n<html><head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>acli - Error</title>\n");
    html.push_str("<style>\n");
    html.push_str(CSS);
    html.push_str("\n</style>\n</head>\n<body>\n<div class=\"container\">\n");
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

// ── CSS ──────────────────────────────────────────────────────────────────────

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

    // ── HTML <img> rewriting ──────────────────────────────────────────────

    #[test]
    fn rewrite_html_img_prepends_media_prefix() {
        let text = r#"<img src="photo.png" alt="pic">"#;
        let result = rewrite_image_urls(text, Path::new("chapter1"));
        assert_eq!(result, r#"<img src="/media/chapter1/photo.png" alt="pic">"#);
    }

    #[test]
    fn rewrite_html_img_root_dir() {
        let text = r#"<img src="photo.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new(""));
        assert_eq!(result, r#"<img src="/media/photo.png" alt="">"#);
    }

    #[test]
    fn rewrite_html_img_leaves_remote_urls() {
        let text = r#"<img src="https://example.com/img.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_html_img_leaves_absolute_paths() {
        let text = r#"<img src="/already/absolute.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    // ── Markdown ![alt](url) rewriting ────────────────────────────────────

    #[test]
    fn rewrite_markdown_img_simple() {
        let text = "![](photo.png)";
        let result = rewrite_image_urls(text, Path::new("chapter1"));
        assert_eq!(result, "![](/media/chapter1/photo.png)");
    }

    #[test]
    fn rewrite_markdown_img_with_alt() {
        let text = "![team logo](team-logos/brewers.png)";
        let result = rewrite_image_urls(text, Path::new("history"));
        assert_eq!(
            result,
            "![team logo](/media/history/team-logos/brewers.png)"
        );
    }

    #[test]
    fn rewrite_markdown_img_root_dir() {
        let text = "![](photo.png)";
        let result = rewrite_image_urls(text, Path::new(""));
        assert_eq!(result, "![](/media/photo.png)");
    }

    #[test]
    fn rewrite_markdown_img_leaves_remote_urls() {
        let text = "![](https://example.com/img.png)";
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_markdown_img_preserves_surrounding_text() {
        let text = "**MLB** (1902)\n![](team-logos/brewers.png)\nsome text";
        let result = rewrite_image_urls(text, Path::new("history"));
        assert!(
            result.contains("![](/media/history/team-logos/brewers.png)"),
            "should rewrite markdown image: {result}"
        );
        assert!(result.contains("**MLB** (1902)"));
        assert!(result.contains("some text"));
    }

    #[test]
    fn rewrite_markdown_img_multiple() {
        let text = "![](a.png) and ![](b.jpg)";
        let result = rewrite_image_urls(text, Path::new("d"));
        assert!(result.contains("/media/d/a.png"), "{result}");
        assert!(result.contains("/media/d/b.jpg"), "{result}");
    }

    // ── Mixed / general ───────────────────────────────────────────────────

    #[test]
    fn rewrite_no_images() {
        let text = "just plain text with no images";
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_leaves_data_uris() {
        let html = r#"<img src="data:image/png;base64,abc" alt="">"#;
        let result = rewrite_image_urls(html, Path::new("sub"));
        assert_eq!(result, html);
    }

    #[test]
    fn rewrite_image_urls_root_dir() {
        let text = r#"<img src="photo.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new(""));
        assert_eq!(result, r#"<img src="/media/photo.png" alt="">"#);
    }

    #[test]
    fn rewrite_image_urls_leaves_remote_urls() {
        let text = r#"<img src="https://example.com/img.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_image_urls_leaves_data_uris() {
        let text = r#"<img src="data:image/png;base64,abc" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_image_urls_leaves_absolute_paths() {
        let text = r#"<img src="/already/absolute.png" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn rewrite_image_urls_preserves_surrounding_text() {
        let text = r#"context <img src="a.png" alt="x"> more text"#;
        let result = rewrite_image_urls(text, Path::new("d"));
        assert_eq!(
            result,
            r#"context <img src="/media/d/a.png" alt="x"> more text"#
        );
    }

    #[test]
    fn rewrite_image_urls_multiple_images() {
        let text = r#"<img src="a.png" alt=""> and <img src="b.jpg" alt="">"#;
        let result = rewrite_image_urls(text, Path::new("pics"));
        assert!(result.contains("/media/pics/a.png"));
        assert!(result.contains("/media/pics/b.jpg"));
    }

    #[test]
    fn rewrite_image_urls_no_images() {
        let text = "just plain text with no images";
        let result = rewrite_image_urls(text, Path::new("sub"));
        assert_eq!(result, text);
    }

    #[test]
    fn percent_decode_basic() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("no%2Fslash"), "no/slash");
        assert_eq!(percent_decode("plain"), "plain");
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
