---
# acli-87ur
title: 'Phase 4: Remove dead code in update-planner-parser'
status: completed
type: task
priority: normal
created_at: 2026-03-24T16:12:55Z
updated_at: 2026-03-24T16:21:37Z
parent: acli-jrua
---

## Goal
Remove DocumentDiff.updated field that is always empty (acknowledged in comment at line 131).

## Tasks
- [x] Remove DocumentDiff.updated field from types.rs
- [x] Remove updated computation in DocumentDiff::compute()
- [x] Remove Operation::Update variant from types.rs
- [x] Remove update handling in DocumentDiff::to_operations()
- [x] Remove update_card from executor::AnkiCollection trait in executor.rs
- [x] Remove update_card implementation from acli/src/adapter.rs
- [x] Update tests in update-planner-parser/tests/diff.rs
- [x] Verify: cargo test -p update-planner-parser && cargo test -p acli

## Summary of Changes

Successfully removed dead code from update-planner-parser:

1. **DocumentDiff.updated field**: Removed the always-empty updated field (with comment acknowledging it was always empty).

2. **Operation::Update variant**: Removed the unused Update variant that was never generated.

3. **update_card method**: Removed update_card from executor::AnkiCollection trait and all implementations (acli/src/adapter.rs, examples/fresh_sync.rs).

4. **Test cleanup**: Removed all assert!(diff.updated.is_empty()) assertions (15 occurrences) and update_count checks from tests.

All tests pass (30 tests in update-planner-parser, 24 tests in acli).
