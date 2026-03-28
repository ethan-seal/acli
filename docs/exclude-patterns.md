# Plan: `exclude` glob patterns in `.acli.toml`

## Usage (end state)

```toml
# .acli.toml
deck = "My Flashcards"
exclude = ["drafts/**", "TODO.md", "archive/*.md"]
```

Patterns are matched against **relative paths** from the working directory (e.g., `drafts/wip.md`, `TODO.md`). Standard glob syntax — `*`, `**`, `?`, `[...]`.

## 1. Add `globset` dependency

**`acli/Cargo.toml`** — add `globset = "0.4.*"` to `[dependencies]`

BurntSushi's crate (same author as `walkdir`). Compiles multiple patterns into one efficient matcher. Well-suited for filtering a list of paths.

## 2. Config struct + merge

**`acli/src/config.rs`**

- Add `pub exclude: Option<Vec<String>>` to `ConfigFile`
- Merge behavior: standard `project.or(user)` — project's list wins if present, user's is the fallback (consistent with every other field in `merge_configs()`)
- Note: `ConfigFile` has `#[serde(deny_unknown_fields)]`, so this field **must** land in the struct before any `.acli.toml` can use it

## 3. Thread through to discovery

**`acli/src/discovery.rs`**

- Change `discover_markdown_files` signature: add `exclude: &[String]`
- Build a `GlobSet` from the patterns up front
- For the recursive `WalkDir` path, use `filter_entry()` to prune excluded **directories** early (e.g. `drafts/**` never descends into `drafts/`)
- For individual files, filter by **relative path** (stripped of source dir prefix) against the `GlobSet`
- If `exclude` is empty, skip building the matcher entirely (zero-cost when unused)

## 4. Plumb through config structs + callers

**`acli/src/sync.rs`**

- Add `pub exclude: Vec<String>` to `SyncConfig` and `ValidationConfig`
- Update `AnkiCli::discover_files()` to accept and forward `exclude` to `discover_markdown_files`
- Note: `serve` currently calls `discover_markdown_files` directly, bypassing `AnkiCli::discover_files()`. Both paths should go through the same `discover_markdown_files` interface so exclude handling is consistent.

**`acli/src/cli.rs`**

- After loading config, read `cfg.exclude` and pass it into `SyncConfig` / `ValidationConfig`
- The `serve` refresh closure captures and forwards the exclude list to `discover_markdown_files`

## 5. Starter config

**`acli/src/config.rs`** — `project_starter_config()`

Add a commented-out example:

```toml
# Glob patterns for markdown files to ignore.
# exclude = ["drafts/**", "TODO.md"]
```

## 6. Tests

- **`config.rs`**: parse with `exclude`, merge (project wins over user), empty/missing exclude. Existing tests (`test_parse_full_config`, `test_merge_project_overrides_user`, `test_merge_user_fills_gaps`, `test_merge_both_empty`) construct exhaustive `ConfigFile` literals — update them with the new field.
- **`discovery.rs`**: excluded files are omitted, non-matching files pass through, `**` recursion works, directory pruning via `filter_entry`
- **Integration**: end-to-end sync skips excluded files

## Files changed (summary)

| File | Change |
|---|---|
| `acli/Cargo.toml` | +`globset` dep |
| `acli/src/config.rs` | +field, merge logic, starter config, tests |
| `acli/src/discovery.rs` | +filtering with `GlobSet`, tests |
| `acli/src/sync.rs` | +field on config structs, update `discover_files()` |
| `acli/src/cli.rs` | plumb `cfg.exclude` into configs + serve |

No CLI flag — this is config-only. A `--exclude` flag could be added later if desired.
