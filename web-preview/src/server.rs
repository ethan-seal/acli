//! HTTP server for card preview.

use crate::renderer::{render_error_page, render_page};
use crate::static_files::serve_static_file;
use crate::{PreviewData, WebPreviewError};
use std::path::PathBuf;

/// Start a blocking web server that serves the card preview page.
///
/// The `refresh` closure is called on **every** page load, so edits to the
/// underlying markdown files are reflected immediately on browser refresh.
///
/// When `static_dir` is `Some`, requests under `/media/…` are served as static
/// files from that directory.  Card fields should use [`crate::rewrite_image_urls`] so
/// that `<img src="…">` paths resolve correctly against this route.
pub fn serve<F>(
    port: u16,
    static_dir: Option<PathBuf>,
    refresh: F,
) -> Result<(), WebPreviewError>
where
    F: Fn() -> Result<PreviewData, String>,
{
    let addr = format!("0.0.0.0:{port}");
    let server = tiny_http::Server::http(&addr).map_err(|e| WebPreviewError::Bind {
        port,
        source: e.into(),
    })?;

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
            let content_type_header = tiny_http::Header::from_bytes(
                &b"Content-Type"[..],
                &b"text/html; charset=utf-8"[..],
            )
            .map_err(|_| WebPreviewError::InvalidHeader)?;
            let response = tiny_http::Response::from_string(&html)
                .with_status_code(status)
                .with_header(content_type_header);
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
