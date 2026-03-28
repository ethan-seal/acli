---
# acli-gt4d
title: 'Phase 2: Deduplicate CLI and sync in acli'
status: completed
type: task
priority: normal
created_at: 2026-03-24T16:12:47Z
updated_at: 2026-03-24T16:16:45Z
parent: acli-jrua
---

## Goal
Extract repeated config resolution and parsing logic in acli crate.

## Tasks
- [x] Extract resolve_config() in cli.rs for Sync/Preview/Validate/Serve branches
- [x] Extract shared discover/parse/validate method in sync.rs
- [x] Simplify run_serve() closure to reuse shared infrastructure
- [x] Verify: cargo test -p acli (24 tests passed)

## Summary of Changes

Successfully deduplicated configuration resolution and parsing logic in acli:

1. **cli.rs**: Extracted resolve_deck_config() helper that combines config loading, deck resolution/validation, and recursive flag resolution. Used by Sync and Preview commands.

2. **sync.rs**: Extracted discover_parse_and_validate() method that combines file discovery, document parsing, and media collision checking. Used by sync() and preview().

3. **cli.rs run_serve()**: Simplified the refresh closure to use AnkiCli::discover_files() instead of manually calling crate::discovery::discover_markdown_files().

All 24 tests in acli pass.
