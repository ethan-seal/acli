---
# acli-dj3m
title: 'Phase 3: Reduce stub boilerplate in anki-wrapper'
status: completed
type: task
priority: normal
created_at: 2026-03-24T16:12:50Z
updated_at: 2026-03-24T16:19:21Z
parent: acli-jrua
---

## Goal
Reduce duplicate error handling in anki-wrapper stubs and real implementation.

## Tasks
- [x] Create not_compiled_error() helper (or macro) for 11 identical stub bodies
- [x] Extract find_notetype() helper for Basic vs BasicReversed lookup
- [x] Extract set_note_fields() helper for field-setting in add_card/update_card
- [x] Verify: cargo check -p anki-wrapper (stubs), cargo test (fakes)

## Summary of Changes

Successfully reduced stub boilerplate in anki-wrapper:

1. **not_compiled_error() helper**: Created generic helper function that replaces 11 identical stub implementations, reducing 143 lines to 14 calls.

2. **find_notetype() helper**: Extracted notetype lookup logic that was duplicated in add_card() between Basic and BasicReversed cases.

3. **set_note_fields() helper**: Extracted field-setting logic shared by add_card() and update_card(), including field count validation.

All tests pass (126 tests across workspace).
