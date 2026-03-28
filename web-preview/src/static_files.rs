//! Static file serving for media assets.

use std::path::{Path, PathBuf};

/// Serve a static file from `static_dir` at the given relative path.
pub(crate) fn serve_static_file(
    request: tiny_http::Request,
    static_dir: &Option<PathBuf>,
    rel_path: &str,
) {
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
    let mut raw: Vec<u8> = Vec::with_capacity(input.len());
    let mut bytes = input.as_bytes().iter().copied();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let hi = bytes.next().and_then(hex_val);
            let lo = bytes.next().and_then(hex_val);
            match (hi, lo) {
                (Some(h), Some(l)) => raw.push(h << 4 | l),
                _ => raw.push(b'%'), // malformed sequence — keep literal
            }
        } else {
            raw.push(b);
        }
    }
    String::from_utf8_lossy(&raw).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_basic() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("no%2Fslash"), "no/slash");
        assert_eq!(percent_decode("plain"), "plain");
    }
}
