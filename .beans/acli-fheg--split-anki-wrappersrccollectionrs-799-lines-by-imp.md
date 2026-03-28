---
# acli-fheg
title: Split anki-wrapper/src/collection.rs (799 lines) by implementation
status: todo
type: task
priority: normal
created_at: 2026-03-25T13:58:49Z
updated_at: 2026-03-25T14:17:39Z
parent: acli-mw5z
---

**Current state:** Single 799-line file with trait definition + stub impl + 2 full implementations (60% over 500-line limit)

**Goal:** Split into separate modules:
- Keep trait definition in `collection.rs`
- `collection/real.rs` - Real Anki backend (lines 210-558)
- `collection/fake.rs` - Fake in-memory impl (lines 560-799)
- `collection/stub.rs` - Stub for missing feature (lines 100-163)

**Location:** `anki-wrapper/src/collection.rs`

**Checklist:**
- [ ] Create `collection/` module directory
- [ ] Extract real implementation
- [ ] Extract fake implementation
- [ ] Extract stub implementation
- [ ] Keep trait in main file
- [ ] Update module exports
- [ ] Verify tests pass
