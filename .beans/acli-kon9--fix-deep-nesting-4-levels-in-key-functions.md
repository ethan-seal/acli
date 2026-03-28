---
# acli-kon9
title: Fix deep nesting (4+ levels) in key functions
status: completed
type: task
priority: normal
created_at: 2026-03-25T13:58:57Z
updated_at: 2026-03-28T18:26:35Z
parent: acli-mw5z
---

**Violations found:**

1. `anki-wrapper/src/collection.rs::get_cards_in_deck()` - 5 levels (lines 421-429)
   - Nested for loop with multiple if-let chains
   - Error handling creates pyramid of doom

2. `web-preview/src/lib.rs::rewrite_markdown_images()` - 4 levels (lines 267-278)
   - Nested if-let branches create deep indentation

3. `acli/src/cli.rs::run_serve()` closure - 5 levels
   - Will be addressed in separate task

**Techniques:**
- Use early returns to reduce nesting
- Extract helper functions for nested operations
- Combine conditional checks where possible
- Use `and_then()` / `ok_or()` for cleaner error handling

**Checklist:**
- [ ] Refactor get_cards_in_deck() nesting
- [ ] Refactor rewrite_markdown_images() nesting
- [ ] Verify tests pass
- [ ] Confirm no nesting exceeds 4 levels
