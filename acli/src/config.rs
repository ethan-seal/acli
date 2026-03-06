//! Configuration file loading for acli.
//!
//! Searches for `.acli.toml` starting from the current directory and walking
//! up to the filesystem root.  The first file found is loaded.  All fields
//! are optional in the file; missing values must be supplied via CLI flags
//! (or will produce an error if required by the subcommand).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::CliError;

/// Name of the config file.
pub const CONFIG_FILENAME: &str = ".acli.toml";

/// Parsed contents of `.acli.toml`.
///
/// All fields are optional — the file may contain only the fields the user
/// cares about.  CLI flags fill in or override any values.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    /// One or more directories containing markdown files.
    /// Accepts either a single string or a list of strings.
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub source: Vec<PathBuf>,

    /// Target Anki deck name.
    pub deck: Option<String>,

    /// Path to the Anki collection database.
    pub collection: Option<PathBuf>,

    /// Path to Anki's `collection.media` directory for media file copying.
    pub media_dir: Option<PathBuf>,

    /// Whether to recurse into subdirectories (default: true).
    pub recursive: Option<bool>,
}

/// Deserialize a field that can be either a single string or a list of
/// strings.  This lets users write `source = "notes/"` instead of
/// `source = ["notes/"]`.
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(PathBuf),
        Multiple(Vec<PathBuf>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multiple(v) => Ok(v),
    }
}

/// Search for `.acli.toml` starting from `start_dir` and walking up.
/// Returns `None` if no config file is found.
pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Load the config file, if one exists.
///
/// Searches from the current working directory upward.  Returns
/// `Ok(None)` if no config file is found.  Returns an error if the file
/// exists but cannot be read or parsed.
pub fn load_config() -> Result<Option<(ConfigFile, PathBuf)>, CliError> {
    let cwd = env::current_dir().map_err(|e| {
        CliError::IoError(std::io::Error::new(
            e.kind(),
            format!("cannot read cwd: {e}"),
        ))
    })?;

    let Some(config_path) = find_config_file(&cwd) else {
        return Ok(None);
    };

    let raw = fs::read_to_string(&config_path)?;
    let config: ConfigFile = toml::from_str(&raw)
        .map_err(|e| CliError::ConfigError(format!("{}: {}", config_path.display(), e)))?;

    Ok(Some((config, config_path)))
}

/// Resolve source paths relative to the directory containing the config file
/// (not the cwd).  This way `source = "notes/"` always means the `notes/`
/// directory next to `.acli.toml`, regardless of where the user runs the
/// command from.
pub fn resolve_source_paths(config_dir: &Path, sources: &[PathBuf]) -> Vec<PathBuf> {
    sources
        .iter()
        .map(|s| {
            if s.is_absolute() {
                s.clone()
            } else {
                config_dir.join(s)
            }
        })
        .collect()
}

/// Starter config content for `acli init`.
pub fn starter_config() -> &'static str {
    r#"# acli configuration
# Docs: https://github.com/ethan-seal/acli

# Source directories containing markdown flashcard files.
# Can be a single string or a list.
source = "."

# Target Anki deck name.
deck = "My Flashcards"

# Path to Anki collection database (optional).
# If omitted, acli uses Anki's default location.
# collection = "/path/to/collection.anki2"

# Path to Anki's collection.media directory (optional).
# Required for syncing images referenced in cards.
# media_dir = "/path/to/Anki2/User 1/collection.media"

# Recurse into subdirectories (default: true).
# recursive = true
"#
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
deck = "Chemistry"
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.deck.as_deref(), Some("Chemistry"));
        assert!(config.source.is_empty());
        assert!(config.collection.is_none());
        assert!(config.media_dir.is_none());
        assert!(config.recursive.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
source = ["notes/", "extra/"]
deck = "Languages"
collection = "/home/user/.local/share/Anki2/User 1/collection.anki2"
media_dir = "/home/user/.local/share/Anki2/User 1/collection.media"
recursive = false
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.source.len(), 2);
        assert_eq!(config.source[0], PathBuf::from("notes/"));
        assert_eq!(config.source[1], PathBuf::from("extra/"));
        assert_eq!(config.deck.as_deref(), Some("Languages"));
        assert!(config.collection.is_some());
        assert!(config.media_dir.is_some());
        assert_eq!(config.recursive, Some(false));
    }

    #[test]
    fn test_parse_single_source_string() {
        let toml = r#"
source = "notes/"
deck = "Test"
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.source, vec![PathBuf::from("notes/")]);
    }

    #[test]
    fn test_parse_unknown_field_is_error() {
        let toml = r#"
deck = "Test"
bogus = true
"#;
        let result = toml::from_str::<ConfigFile>(toml);
        assert!(result.is_err(), "unknown fields should be rejected");
    }

    #[test]
    fn test_find_config_walks_up() {
        // Create a temp directory tree: root/.acli.toml, root/sub/
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(root.join(CONFIG_FILENAME), "deck = \"Found\"").unwrap();

        // Searching from sub/ should find root/.acli.toml
        let found = find_config_file(&sub);
        assert_eq!(found, Some(root.join(CONFIG_FILENAME)));
    }

    #[test]
    fn test_find_config_returns_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_config_file(tmp.path()).is_none());
    }

    #[test]
    fn test_resolve_relative_paths() {
        let config_dir = Path::new("/project");
        let sources = vec![PathBuf::from("notes/"), PathBuf::from("/abs/path")];
        let resolved = resolve_source_paths(config_dir, &sources);
        assert_eq!(resolved[0], PathBuf::from("/project/notes/"));
        assert_eq!(resolved[1], PathBuf::from("/abs/path"));
    }
}
