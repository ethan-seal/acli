---
# acli-mrlx
title: Refactor run_serve() function (87 lines) in acli/src/cli.rs
status: todo
type: task
priority: normal
created_at: 2026-03-25T13:58:50Z
updated_at: 2026-03-25T14:17:39Z
parent: acli-mw5z
---

**Current state:** 87-line function (exceeds 80-line guideline) handling file discovery, parsing, card conversion, image rewriting, and server setup

**Location:** `acli/src/cli.rs` lines 313-399

**Issues:**
- Inner closure `refresh` is 64 lines of complex logic
- 5 levels of nesting in inner conditions
- Multiple concerns mixed together

**Goal:** Extract helper functions:
- `discover_card_files()` - File discovery logic
- `parse_and_convert_cards()` - Parsing loop and card conversion
- `setup_card_server()` - Server setup

**Checklist:**
- [ ] Extract file discovery to helper
- [ ] Extract parsing/conversion to helper
- [ ] Reduce nesting in refresh closure
- [ ] Simplify main function flow
- [ ] Verify serve command works
