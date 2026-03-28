---
# acli-gnbr
title: Remove code duplication patterns (3+ occurrences)
status: todo
type: task
priority: normal
created_at: 2026-03-25T13:59:00Z
updated_at: 2026-03-25T14:17:39Z
parent: acli-mw5z
---

**Duplication patterns found:**

1. **Error handling pattern** (15+ occurrences in `anki-wrapper/src/collection.rs`):
   `.map_err(|e| AnkiWrapperError::AnkiError(format!("Failed to X: {}", e)))?`
   - Extract to helper macro or function

2. **Card type conversions** (duplicated in `cli.rs` and `output.rs`):
   - Lines 366-372 in `acli/src/cli.rs`
   - Lines 104-108 in `acli/src/output.rs`
   - Extract to shared utility function

3. **Deck config creation** (6+ occurrences in `adapter.rs`):
   - Lines 46-48, 55-57, 83-85, 145-147, 154-156, 167-169
   - Extract to helper function

**Checklist:**
- [ ] Extract error handling pattern
- [ ] Extract card type conversion to utility
- [ ] Extract deck config creation helper
- [ ] Replace all duplicated instances
- [ ] Verify tests pass
