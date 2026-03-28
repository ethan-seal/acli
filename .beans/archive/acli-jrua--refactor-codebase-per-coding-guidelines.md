---
# acli-jrua
title: Refactor codebase per coding guidelines
status: completed
type: epic
priority: normal
created_at: 2026-03-24T16:12:29Z
updated_at: 2026-03-24T16:22:32Z
---

Apply CLAUDE.md coding guidelines to reduce complexity across the codebase. This is a pure refactoring with no functional changes - all tests must pass after each phase.



## Final Summary

All 5 phases completed successfully:

**Phase 1**: Split doc-parser/src/parser.rs (1301 lines) into 7 focused files  
**Phase 2**: Extracted config resolution and parsing helpers in acli  
**Phase 3**: Reduced anki-wrapper stub boilerplate (143 lines → 14 calls)  
**Phase 4**: Removed dead code (DocumentDiff.updated, Operation::Update)  
**Phase 5**: Verified all 126 tests pass, clippy clean

All changes are pure refactors with no functional behavior changes.
