---
# acli-djha
title: 'Phase 1: Split doc-parser/src/parser.rs into module'
status: completed
type: task
created_at: 2026-03-24T16:12:43Z
updated_at: 2026-03-24T16:12:43Z
parent: acli-jrua
---

## Goal
Split the 1301-line parser.rs into a focused module directory.

## Tasks
- [x] Create parser/ module directory
- [x] Create extraction.rs with ExtractionResult type and build_residual helper
- [x] Create pipe_table.rs with data-driven parse_col_header
- [x] Create template.rs
- [x] Create sequence.rs
- [x] Create block.rs
- [x] Create inline.rs with ParseState
- [x] Create mod.rs with DocumentParser impl
- [x] Delete old parser.rs
- [x] Verify all tests pass

## Summary of Changes
Successfully split parser.rs (1301 lines) into 7 focused files in parser/ module:
- extraction.rs: Shared ExtractionResult type and build_residual helper (30 lines)
- pipe_table.rs: Pipe table extraction with data-driven parse_col_header and emit_pipe_cards helper (213 lines)
- template.rs: Template expansion logic (175 lines)
- sequence.rs: Sequence card extraction (130 lines)
- block.rs: Block card extraction (100 lines)
- inline.rs: Inline parsing with ParseState (275 lines)
- mod.rs: DocumentParser trait and MarkdownParser impl with tests (200 lines)

All extractors now return unified ExtractionResult type. All 10 tests pass with no warnings.
