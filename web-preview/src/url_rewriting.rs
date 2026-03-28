//! Image URL rewriting for card preview.

use std::path::Path;

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
            let url_start = bracket_paren + "](".len();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
}
