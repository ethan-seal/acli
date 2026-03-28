//! Configuration file loading for acli.
//!
//! Two config files are supported, loaded and merged in order:
//!
//! 1. **User config** — `$XDG_CONFIG_HOME/acli/config.toml` (falls back to
//!    `~/.config/acli/config.toml`).  Typically contains machine-specific
//!    Anki paths (`collection`, `media_dir`).
//!
//! 2. **Project config** — `.acli.toml` searched from the current directory
//!    upward.  Typically contains `deck` and any project-specific overrides.
//!
//! For each field, the project config overrides the user config, and CLI
//! flags override both.  Missing values must be supplied via CLI flags (or
//! will produce an error if required by the subcommand).
//!
//! Source directories are NOT configurable — acli always syncs the current
//! working directory.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Name of the project-local config file.
pub const PROJECT_CONFIG_FILENAME: &str = ".acli.toml";

/// Directory name inside XDG config for the user-level config.
const USER_CONFIG_DIR: &str = "acli";

/// Filename of the user-level config.
const USER_CONFIG_FILENAME: &str = "config.toml";

/// Parsed config file contents.
///
/// The same struct is used for both user and project configs — all fields
/// are optional, and the two are merged before use.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    /// Target Anki deck name.
    pub deck: Option<String>,

    /// Path to the Anki collection database.
    pub collection: Option<PathBuf>,

    /// Path to Anki's `collection.media` directory for media file copying.
    pub media_dir: Option<PathBuf>,

    /// Whether to recurse into subdirectories (default: true).
    pub recursive: Option<bool>,
}

// ── Config loading ────────────────────────────────────────────────────────────

/// Result of loading and merging both config files.
pub struct ResolvedConfig {
    /// Merged configuration (user + project).
    pub config: ConfigFile,
    /// Path to the user config file, if one was loaded.
    pub user_path: Option<PathBuf>,
    /// Path to the project config file, if one was loaded.
    pub project_path: Option<PathBuf>,
}

/// Load both config files (if they exist) and merge them.
///
/// Returns the merged config plus the paths of whichever files were found.
/// Returns an error if a config file exists but cannot be read or parsed.
pub fn load_config() -> Result<ResolvedConfig> {
    let user = load_file(&user_config_path())?;
    let user_path = user.as_ref().map(|_| user_config_path());
    let user_cfg = user.unwrap_or_default();

    let cwd = env::current_dir().context("cannot read cwd")?;
    let project_file = find_project_config(&cwd);
    let project_path = project_file.clone();
    let project_cfg = match project_file {
        Some(ref path) => load_file(path)?.unwrap_or_default(),
        None => ConfigFile::default(),
    };

    Ok(ResolvedConfig {
        config: merge_configs(&user_cfg, &project_cfg),
        user_path,
        project_path,
    })
}

/// Load and parse a single config file.  Returns `Ok(None)` if the file
/// does not exist.
fn load_file(path: &Path) -> Result<Option<ConfigFile>> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let config: ConfigFile = toml::from_str(&raw)
        .with_context(|| format!("{}: failed to parse config", path.display()))?;
    Ok(Some(config))
}

// ── Project config ────────────────────────────────────────────────────────────

/// Search for `.acli.toml` starting from `start_dir` and walking up.
/// Returns `None` if no config file is found.
pub fn find_project_config(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(PROJECT_CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

// ── User config ───────────────────────────────────────────────────────────────

/// Canonical path for the user-level config file.
///
/// Uses `$XDG_CONFIG_HOME/acli/config.toml` if `XDG_CONFIG_HOME` is set,
/// otherwise falls back to `$HOME/.config/acli/config.toml`.
pub fn user_config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join(USER_CONFIG_DIR).join(USER_CONFIG_FILENAME)
}

// ── Merging ───────────────────────────────────────────────────────────────────

/// Merge two configs.  For each field, `project` wins if it provides a
/// value, otherwise `user` provides the fallback.
pub fn merge_configs(user: &ConfigFile, project: &ConfigFile) -> ConfigFile {
    ConfigFile {
        deck: project.deck.clone().or_else(|| user.deck.clone()),
        collection: project
            .collection
            .clone()
            .or_else(|| user.collection.clone()),
        media_dir: project.media_dir.clone().or_else(|| user.media_dir.clone()),
        recursive: project.recursive.or(user.recursive),
    }
}

// ── Starter configs ───────────────────────────────────────────────────────────

/// Starter content for a project-level `.acli.toml`.
pub fn project_starter_config() -> &'static str {
    r#"# acli project configuration
# Docs: https://github.com/ethan-seal/acli

# Target Anki deck name.
deck = "My Flashcards"

# Recurse into subdirectories (default: true).
# recursive = true
"#
}

/// Starter content for the user-level `config.toml`.
pub fn user_starter_config() -> &'static str {
    r#"# acli user configuration
# Docs: https://github.com/ethan-seal/acli
#
# Machine-specific settings shared across all projects.
# Project-level .acli.toml overrides values set here.

# Path to Anki collection database.
# collection = "/home/user/.local/share/Anki2/User 1/collection.anki2"

# Path to Anki's collection.media directory.
# Required for syncing images referenced in cards.
# media_dir = "/home/user/.local/share/Anki2/User 1/collection.media"
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
        assert!(config.collection.is_none());
        assert!(config.media_dir.is_none());
        assert!(config.recursive.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
deck = "Languages"
collection = "/home/user/.local/share/Anki2/User 1/collection.anki2"
media_dir = "/home/user/.local/share/Anki2/User 1/collection.media"
recursive = false
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.deck.as_deref(), Some("Languages"));
        assert!(config.collection.is_some());
        assert!(config.media_dir.is_some());
        assert_eq!(config.recursive, Some(false));
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
    fn test_find_project_config_walks_up() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(root.join(PROJECT_CONFIG_FILENAME), "deck = \"Found\"").unwrap();

        let found = find_project_config(&sub);
        assert_eq!(found, Some(root.join(PROJECT_CONFIG_FILENAME)));
    }

    #[test]
    fn test_find_project_config_returns_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_project_config(tmp.path()).is_none());
    }

    #[test]
    fn test_merge_project_overrides_user() {
        let user = ConfigFile {
            deck: Some("UserDeck".into()),
            collection: Some("/user/col".into()),
            media_dir: Some("/user/media".into()),
            recursive: Some(false),
        };
        let project = ConfigFile {
            deck: Some("ProjectDeck".into()),
            collection: None,
            media_dir: None,
            recursive: None,
        };
        let merged = merge_configs(&user, &project);
        assert_eq!(merged.deck.as_deref(), Some("ProjectDeck"));
        assert_eq!(merged.collection, Some("/user/col".into()));
        assert_eq!(merged.media_dir, Some("/user/media".into()));
        assert_eq!(merged.recursive, Some(false));
    }

    #[test]
    fn test_merge_user_fills_gaps() {
        let user = ConfigFile {
            deck: None,
            collection: Some("/user/col".into()),
            media_dir: None,
            recursive: None,
        };
        let project = ConfigFile {
            deck: Some("MyDeck".into()),
            collection: None,
            media_dir: None,
            recursive: None,
        };
        let merged = merge_configs(&user, &project);
        assert_eq!(merged.deck.as_deref(), Some("MyDeck"));
        assert_eq!(merged.collection, Some("/user/col".into()));
        assert!(merged.media_dir.is_none());
        assert!(merged.recursive.is_none());
    }

    #[test]
    fn test_merge_both_empty() {
        let merged = merge_configs(&ConfigFile::default(), &ConfigFile::default());
        assert_eq!(merged, ConfigFile::default());
    }

    #[test]
    fn test_user_config_path_uses_xdg() {
        // Save and override XDG_CONFIG_HOME.
        let prev = env::var_os("XDG_CONFIG_HOME");
        // SAFETY: test is single-threaded for this env var.
        unsafe { env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test") };
        let path = user_config_path();
        match prev {
            Some(v) => unsafe { env::set_var("XDG_CONFIG_HOME", v) },
            None => unsafe { env::remove_var("XDG_CONFIG_HOME") },
        }
        assert_eq!(path, PathBuf::from("/tmp/xdg-test/acli/config.toml"));
    }

    #[test]
    fn test_user_config_path_falls_back_to_home() {
        let prev_xdg = env::var_os("XDG_CONFIG_HOME");
        let prev_home = env::var_os("HOME");
        // SAFETY: test is single-threaded for these env vars.
        unsafe { env::remove_var("XDG_CONFIG_HOME") };
        unsafe { env::set_var("HOME", "/tmp/fakehome") };
        let path = user_config_path();
        match prev_xdg {
            Some(v) => unsafe { env::set_var("XDG_CONFIG_HOME", v) },
            None => unsafe { env::remove_var("XDG_CONFIG_HOME") },
        }
        match prev_home {
            Some(v) => unsafe { env::set_var("HOME", v) },
            None => unsafe { env::remove_var("HOME") },
        }
        assert_eq!(
            path,
            PathBuf::from("/tmp/fakehome/.config/acli/config.toml")
        );
    }
}
