---
# acli-hyj6
title: Split web-preview/src/lib.rs (920 lines) into focused modules
status: completed
type: task
priority: high
created_at: 2026-03-25T13:58:42Z
updated_at: 2026-03-25T14:21:07Z
parent: acli-mw5z
---

**Current state:** Single 920-line file combining server, routing, rendering, CSS, image rewriting, and static files (84% over 500-line limit)

**Goal:** Split into focused modules:
- `server.rs` - HTTP server and routing (lines 74-125, 98-120)
- `renderer.rs` - HTML/CSS rendering (lines 327-613)
- `url_rewriting.rs` - Image URL rewriting (lines 215-288)
- `static_files.rs` - Static file serving (lines 128-170)

**Location:** `web-preview/src/lib.rs`

**Checklist:**
- [x] Create module structure
- [x] Extract server routing logic
- [x] Extract HTML/CSS rendering
- [x] Extract URL rewriting
- [x] Extract static file serving
- [x] Update tests
- [x] Verify all functionality works


## Summary of Changes

Successfully split the 920-line  into focused modules:

**New file structure:**
- `lib.rs` (68 lines) - Public types and API re-exports
- `server.rs` (67 lines) - HTTP server and routing logic
- `static_files.rs` (105 lines) - Static file serving with security checks
- `url_rewriting.rs` (249 lines) - Image URL rewriting for both HTML and Markdown
- `renderer.rs` (458 lines) - Page rendering, HTML generation, and CSS

**Results:**
- ✅ Main lib.rs reduced from 920 → 68 lines (93% reduction)
- ✅ All modules under 500-line guideline (largest is renderer.rs at 458 lines)
- ✅ All 31 tests passing
- ✅ Main acli crate compiles successfully
- ✅ Each module has clear, focused responsibility
- ✅ Public API unchanged - backward compatible

**Guideline compliance:**
- Was: 920 lines (84% over 500-line limit)
- Now: Largest file is 458 lines (9% under limit)
