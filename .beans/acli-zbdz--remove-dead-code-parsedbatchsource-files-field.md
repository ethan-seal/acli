---
# acli-zbdz
title: 'Remove dead code: ParsedBatch::source_files field'
status: completed
type: task
priority: low
created_at: 2026-03-25T13:59:04Z
updated_at: 2026-03-28T18:37:10Z
parent: acli-mw5z
---

**Current state:** Field marked with `#[allow(dead_code)]` in `acli/src/sync.rs` line 410

**Location:** `acli/src/sync.rs` lines 407-413

**Issue:** 
- Field is populated (line 336) but never used in public API
- Violates "no dead code" guideline

**Action:**
- Remove `source_files` field from `ParsedBatch` struct
- Remove population of this field (line 336)
- Verify tests pass

**Checklist:**
- [ ] Remove field from struct definition
- [ ] Remove field population
- [ ] Verify compilation succeeds
- [ ] Verify tests pass
