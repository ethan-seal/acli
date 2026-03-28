use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use walkdir::WalkDir;

pub fn discover_markdown_files(
    sources: &[PathBuf],
    recursive: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for source in sources {
        if !source.exists() {
            bail!("invalid path: {}", source.display());
        }

        if source.is_file() {
            if is_markdown_file(source) {
                files.push(source.clone());
            }
            continue;
        }

        if source.is_dir() {
            discover_in_directory(source, recursive, &mut files)?;
            continue;
        }

        bail!("invalid path: {}", source.display());
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn discover_in_directory(
    directory: &Path,
    recursive: bool,
    output: &mut Vec<PathBuf>,
) -> Result<()> {
    if recursive {
        for entry in WalkDir::new(directory) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_markdown_file(path) {
                output.push(path.to_path_buf());
            }
        }
    } else {
        let entries = fs::read_dir(directory)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_markdown_file(&path) {
                output.push(path);
            }
        }
    }

    Ok(())
}

pub fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}
