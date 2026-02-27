//! Media file copying utilities for syncing to Anki's collection.media folder.

use std::io;
use std::path::{Path, PathBuf};

use doc_parser::MediaReference;

/// Report of what happened during a media copy operation.
#[derive(Debug, Default, PartialEq)]
pub struct CopyReport {
    /// Number of files successfully copied to the destination.
    pub copied: usize,
    /// Number of files skipped because they already existed in the destination.
    pub skipped: usize,
    /// Paths of source files that were not found (warned but not failed).
    pub missing: Vec<PathBuf>,
}

/// Copy media files from source locations to Anki's media folder.
///
/// - `document_dir`: directory containing the source Markdown file (for resolving relative paths)
/// - `media_refs`: references extracted by the parser
/// - `anki_media_dir`: path to Anki's `collection.media` folder
///
/// **Behaviour:**
/// - Resolves `source_path` relative to `document_dir`
/// - Skips files that already exist at the destination (no overwrite)
/// - Warns on missing source files but does not fail — records them in `CopyReport::missing`
/// - Copies to `anki_media_dir / target_name`
///
/// Returns a `CopyReport` on success. Only returns `Err` for unexpected I/O errors
/// (not for missing source files — those are recorded in `CopyReport::missing`).
pub fn copy_media_to_anki(
    document_dir: &Path,
    media_refs: &[MediaReference],
    anki_media_dir: &Path,
) -> Result<CopyReport, io::Error> {
    let mut report = CopyReport::default();

    for media_ref in media_refs {
        let src = document_dir.join(&media_ref.source_path);
        let dst = anki_media_dir.join(&media_ref.target_name);

        if dst.exists() {
            report.skipped += 1;
            continue;
        }

        if !src.exists() {
            eprintln!("Warning: media file not found: {}", src.display());
            report.missing.push(src);
            continue;
        }

        std::fs::copy(&src, &dst)?;
        report.copied += 1;
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_parser::MediaReference;
    use tempfile::TempDir;

    fn make_ref(source_path: &str, target_name: &str) -> MediaReference {
        MediaReference::new(source_path.to_string(), target_name.to_string(), None, 0).unwrap()
    }

    #[test]
    fn test_copies_file_to_destination() {
        let doc_dir = TempDir::new().unwrap();
        let anki_dir = TempDir::new().unwrap();

        // Create source file
        let src = doc_dir.path().join("image.jpg");
        std::fs::write(&src, b"fake image data").unwrap();

        let refs = vec![make_ref("image.jpg", "image.jpg")];
        let report = copy_media_to_anki(doc_dir.path(), &refs, anki_dir.path()).unwrap();

        assert_eq!(report.copied, 1);
        assert_eq!(report.skipped, 0);
        assert!(report.missing.is_empty());
        assert!(anki_dir.path().join("image.jpg").exists());
    }

    #[test]
    fn test_skips_already_present_file() {
        let doc_dir = TempDir::new().unwrap();
        let anki_dir = TempDir::new().unwrap();

        // Create source file
        std::fs::write(doc_dir.path().join("image.jpg"), b"source").unwrap();
        // Pre-populate destination
        std::fs::write(anki_dir.path().join("image.jpg"), b"existing").unwrap();

        let refs = vec![make_ref("image.jpg", "image.jpg")];
        let report = copy_media_to_anki(doc_dir.path(), &refs, anki_dir.path()).unwrap();

        assert_eq!(report.copied, 0);
        assert_eq!(report.skipped, 1);
        assert!(report.missing.is_empty());
        // Destination should still have the original content
        let content = std::fs::read(anki_dir.path().join("image.jpg")).unwrap();
        assert_eq!(content, b"existing");
    }

    #[test]
    fn test_warns_on_missing_source_file_does_not_fail() {
        let doc_dir = TempDir::new().unwrap();
        let anki_dir = TempDir::new().unwrap();

        // Source file does NOT exist
        let refs = vec![make_ref("missing.jpg", "missing.jpg")];
        let report = copy_media_to_anki(doc_dir.path(), &refs, anki_dir.path()).unwrap();

        assert_eq!(report.copied, 0);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.missing.len(), 1);
        // Nothing should have been created in destination
        assert!(!anki_dir.path().join("missing.jpg").exists());
    }

    #[test]
    fn test_resolves_relative_paths_from_document_dir() {
        let doc_dir = TempDir::new().unwrap();
        let anki_dir = TempDir::new().unwrap();

        // Create source file in a subdir relative to doc_dir
        let subdir = doc_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("photo.png"), b"photo data").unwrap();

        let refs = vec![make_ref("subdir/photo.png", "photo.png")];
        let report = copy_media_to_anki(doc_dir.path(), &refs, anki_dir.path()).unwrap();

        assert_eq!(report.copied, 1);
        assert!(anki_dir.path().join("photo.png").exists());
    }

    #[test]
    fn test_reports_counts_correctly() {
        let doc_dir = TempDir::new().unwrap();
        let anki_dir = TempDir::new().unwrap();

        // Create two source files
        std::fs::write(doc_dir.path().join("a.jpg"), b"a").unwrap();
        std::fs::write(doc_dir.path().join("b.jpg"), b"b").unwrap();
        // Pre-populate one destination file
        std::fs::write(anki_dir.path().join("b.jpg"), b"existing b").unwrap();

        let refs = vec![
            make_ref("a.jpg", "a.jpg"), // will be copied
            make_ref("b.jpg", "b.jpg"), // will be skipped
            make_ref("c.jpg", "c.jpg"), // missing
        ];
        let report = copy_media_to_anki(doc_dir.path(), &refs, anki_dir.path()).unwrap();

        assert_eq!(report.copied, 1, "copied");
        assert_eq!(report.skipped, 1, "skipped");
        assert_eq!(report.missing.len(), 1, "missing");
    }
}
