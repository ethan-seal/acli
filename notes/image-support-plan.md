# Image Support Implementation Plan

## Overview

Add support for image-based flashcards to enable converting Art.xls and similar visual learning materials to the Markdown card format.

## Goals

1. Parse Markdown image syntax `![alt](path)` within card definitions
2. Convert to Anki-compatible HTML `<img>` tags
3. Track media file references for copying during sync
4. Copy media files to Anki's collection.media folder

## Non-Goals (for this iteration)

- Audio/video support (future extension using same pattern)
- Embedded base64 images
- Remote URLs (only local file references)

---

## Architecture Decision: AST-Based Parser

The current parser is ~120 lines of manual line-based string splitting (`split_once("->")`, leading-space counting, etc.). `pulldown-cmark` v0.9 is already declared as a dependency but unused — the README notes this migration was always planned.

Rather than bolting regex-based image extraction onto the line-based parser, we will **rewrite the parser to walk a `pulldown-cmark` AST**. This gives us:

1. **Image support for free** — `pulldown-cmark` emits `Event::Start(Tag::Image)` natively
2. **Correct Markdown handling** — no more edge cases with arrows inside code spans, escaped characters, etc.
3. **Single-pass processing** — parse once into AST, walk once to extract cards + media refs
4. **Foundation for future features** — code blocks, tables, HTML rendering for Anki all become straightforward future additions

### Parsing Strategy

Walk the `pulldown-cmark` event stream. The card-detection logic stays the same conceptually:

1. Track list nesting depth via `Event::Start(Tag::List)` / `Event::Start(Tag::Item)` events
2. Within each list item, accumulate inline content (text, images, emphasis, etc.)
3. Scan accumulated text for `->` or `<->` arrow delimiters to split into question/answer
4. Render each side to HTML via `pulldown-cmark::html::push_html()` on the sub-events
5. Collect `MediaReference` entries from any `Tag::Image` events encountered

This is a single pass over the event stream producing both `Vec<Card>` and `Vec<MediaReference>`.

---

## Decisions

### Filename normalization

**Strategy: keep the original filename, fail on invalid characters.**

Anki's `collection.media` folder is a flat namespace. Our normalization rules:
- Strip any directory components — `dir/sub/image.jpg` becomes `image.jpg` as the `target_name`
- Reject filenames containing characters illegal in Anki media: `[`, `]`, `"`, `*`, `:`, `?`, `|`, `\\`, and control characters
- **Fail at parse time** if a filename contains these characters, with a clear error pointing to the line and suggesting a rename
- No hashing, no silent renaming — the user sees exactly the filename they wrote

This is the simplest correct approach. If two different source files resolve to the same `target_name` (e.g., `art/mona.jpg` and `photos/mona.jpg`), that's a **hard error** at sync time with a message explaining the collision.

### Path resolution

**Resolved: relative to the document file.**

The document path is passed separately into the media copy function (not stored on `ParsedDocument`). The parser only sees the relative path string from the Markdown; resolution happens at sync time.

### Missing images at sync time

**Warn and continue.** Log a warning with the missing path, skip copying that file, but still create the card. The card will show a broken image in Anki, which is a clear signal to the user.

### Filename collisions

**Hard error.** If two documents reference different source files that would produce the same `target_name`, the sync fails with an error listing both source paths and suggesting a rename.

### Re-sync with changed images

**Skip if file already exists** in `collection.media`. For MVP this is fast and simple. If a user updates an image, they can delete the old copy from `collection.media` to force a re-copy. Optimizing this with checksums is a future enhancement.

---

## Phase 0: Parser Rewrite (pulldown-cmark)

This is a prerequisite for image support and replaces the current line-based parser.

### Task 0.1: Rewrite MarkdownParser::parse() using pulldown-cmark

Replace the line-based parser with a `pulldown-cmark` event walker.

**File:** `doc-parser/src/parser.rs`

**Algorithm:**
```rust
fn parse(&self, markdown: &str) -> Result<ParsedDocument, ParseError> {
    let parser = pulldown_cmark::Parser::new(markdown);
    
    // State machine:
    // - Track list depth via Start(List)/End(List) events
    // - Track current item content via Start(Item)/End(Item)
    // - Within items, accumulate events into a buffer
    // - On End(Item), scan buffer for arrow delimiters
    // - Split into LHS/RHS event subsequences
    // - Extract raw text for each side (preserving inline formatting as-is)
    // - Replace image syntax with <img> tags
    // - Non-leaf items (those containing sub-lists) become context parents
}
```

**Key design point:** Arrow detection (`->`, `<->`) happens on the *text content* of a list item. Card fields remain as raw text (same as today), except that `![alt](path)` image syntax gets replaced with `<img>` tags. This means:
- Existing text-only cards produce identical `Card.fields` and identical `CardId` hashes — no migration needed
- Image cards are all new, so no hash collision concern
- HTML rendering of full card content (bold, italic, etc.) is a separate future concern, not part of this work

**Tests (must pass all existing tests with identical output):**
- `test_basic_card` — `"One -> 1"` → fields `["One -> ?", "1"]` (unchanged)
- `test_bidirectional_card` — `"One <-> 1"` → fields `["One <-> ?", "1"]` (unchanged)
- `test_nested_context_list` — parent context propagation (unchanged)
- All CardId tests (unchanged, they don't touch the parser)

**New tests for correct Markdown handling:**
- Arrow inside code span is NOT treated as delimiter: `` `a -> b` -> answer ``
- Inline formatting is preserved as-is in fields: `**bold** -> answer` → fields `["**bold** -> ?", "answer"]`

### Task 0.2: Verify backward compatibility

Run the full test suite and any integration tests to confirm the rewrite produces identical `ParsedDocument` output for all existing inputs. Since card fields remain as raw text, `CardId` hashes are unchanged and no migration is needed.

---

## Phase 1: Image Support in Parser

With the pulldown-cmark parser in place, image support is a natural extension.

### Task 1.1: Add MediaReference type to doc-parser

**File:** `doc-parser/src/types.rs`

```rust
/// Reference to a media file discovered during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaReference {
    /// Path to source file as written in the Markdown (may include directories).
    pub source_path: String,
    /// Filename to use in Anki's collection.media (just the filename component).
    pub target_name: String,
    /// Alt text from Markdown, if any.
    pub alt_text: Option<String>,
}
```

**Update ParsedDocument:**
```rust
pub struct ParsedDocument {
    pub cards: Vec<Card>,
    pub media: Vec<MediaReference>,  // NEW
    pub source_path: Option<String>,
}
```

**Validation (at parse time):**
- `target_name` must not contain `[ ] " * : ? | \` or control characters — return `ParseError` if it does
- `source_path` must not be empty

**Tests:**
- Unit test for MediaReference construction
- Test that ParsedDocument::default() has empty media vec
- Validation rejects `image[1].jpg`, `file"name.png`, `img:2.jpg`

### Task 1.2: Extract MediaReferences from pulldown-cmark Image events

In the AST walker from Phase 0, when encountering `Event::Start(Tag::Image(_, url, _))`:

1. Create a `MediaReference` with `source_path = url` and `target_name = filename component of url`
2. Validate the target_name
3. Add to the document's media vec
4. The image event is already rendered to `<img>` by `push_html()` — no extra work needed

**Deduplication:** Collect into a `HashSet<MediaReference>` (or dedup after collecting) so the same image referenced by multiple cards only appears once.

**Tests:**
- `![](image.jpg) -> answer` — card front contains `<img ...>`, media has one ref
- `question -> ![Alt](dir/image.png)` — card answer contains `<img ...>`, media ref has `source_path="dir/image.png"`, `target_name="image.png"`, `alt_text=Some("Alt")`
- `![](a.jpg) <-> ![](b.jpg)` — bidirectional card, two media refs
- Two cards referencing same image — one media ref (deduped)
- Nested context with image in child card
- Card with no images — empty media vec, card works normally

---

## Phase 2: CLI Media Handling

### Task 2.1: Add media copy functionality to acli

**File:** new `src/media.rs`

```rust
/// Copy media files from source locations to Anki's media folder.
///
/// `document_dir`: directory containing the source Markdown file (for resolving relative paths)
/// `media_refs`: references extracted by the parser
/// `anki_media_dir`: path to Anki's collection.media folder
///
/// Skips files that already exist in the destination.
/// Warns on missing source files but does not fail.
/// Fails on filename collisions (two different source files with the same target_name).
fn copy_media_to_anki(
    document_dir: &Path,
    media_refs: &[MediaReference],
    anki_media_dir: &Path,
) -> Result<CopyReport, Error> {
    for ref in media_refs {
        let src = document_dir.join(&ref.source_path);
        let dst = anki_media_dir.join(&ref.target_name);
        
        if dst.exists() {
            // Skip — already present
            continue;
        }
        
        if !src.exists() {
            // Warn and continue
            warn!("Media file not found: {}", src.display());
            continue;
        }
        
        fs::copy(&src, &dst)?;
    }
}
```

**CopyReport** tracks: files copied, files skipped (already present), files missing (warned).

**Tests:**
- Copies file to destination
- Skips already-present files (does not overwrite)
- Warns on missing source files, does not fail
- Resolves relative paths from document directory
- Reports counts correctly

### Task 2.2: Detect filename collisions across documents

When syncing multiple documents, collect all `MediaReference`s and check for collisions: two different `source_path` values producing the same `target_name`.

```rust
fn check_media_collisions(all_refs: &[MediaReference]) -> Result<(), Error> {
    // Group by target_name
    // If any target_name has multiple distinct source_paths, fail with descriptive error
}
```

**Tests:**
- Same source_path from two documents — OK (dedup)
- Different source_paths, same target_name — hard error with both paths in message

### Task 2.3: Integrate media handling into sync workflow

Modify the sync command to:
1. Collect all MediaReferences from all parsed documents
2. Check for filename collisions (fail fast)
3. Copy media files before creating/updating cards
4. Report media copy results (copied N, skipped M, warned K)

**Tests:**
- Full sync with image cards (integration test)
- Sync skips already-present media files
- Clear error message for missing images
- Clear error message for filename collisions

---

## Phase 3: anki-wrapper Media API (Optional, deferred)

Direct file copying to `collection.media` is sufficient for MVP. If we later need programmatic access to Anki's `MediaManager` (e.g., for garbage collection or integrity checks), add:

```rust
trait AnkiCollection {
    fn add_media_file(&mut self, name: &str, data: &[u8]) -> Result<String, Error>;
    fn media_file_exists(&self, name: &str) -> Result<bool, Error>;
}
```

Not planned for this iteration.

---

## Testing Strategy

### Unit Tests (doc-parser)
- All existing tests pass with identical output (no CardId changes)
- Image syntax produces `<img>` tags in card fields and correct MediaReferences
- Filename validation rejects illegal characters
- Deduplication of media refs
- Arrows inside code spans are not treated as delimiters

### Integration Tests (acli)
- Full sync with image cards
- Media file copying and skip-if-exists
- Missing file warnings
- Filename collision errors

### E2E Demo
- Add an Art cards example to e2e demo
- Verify images appear correctly in Anki

---

## Migration: Converting Art.xls

Script to convert the existing Art.xls to Markdown using Bun:

```typescript
#!/usr/bin/env bun

/**
 * Convert Art.xls to Markdown card format.
 *
 * Setup: bun add xlsx
 * Usage: bun convert_art.ts Art.xls > art-cards.md
 */

import { readFileSync } from "node:fs";
import * as XLSX from "xlsx";

const xlsPath = Bun.argv[2];
if (!xlsPath) {
  console.error("Usage: bun convert_art.ts <xls_file>");
  process.exit(1);
}

const data = readFileSync(xlsPath);
const workbook = XLSX.read(data);
const sheet = workbook.Sheets[workbook.SheetNames[0]];
const rows: string[][] = XLSX.utils.sheet_to_json(sheet, { header: 1 });

for (const row of rows) {
  const imgPath = row[0] ?? "";
  const info = row[1] ?? "";

  const parts = String(info).split("\n", 2);
  const title = parts[0]?.trim() || "Unknown";
  const artist = parts[1]?.trim() || "Unknown artist";

  console.log(`- ![](${imgPath}) -> ${title} by ${artist}`);
}
```

---

## Task Summary

| ID | Task | Priority | Est. Size | Depends On |
|----|------|----------|-----------|------------|
| 0.1 | Rewrite parser with pulldown-cmark AST | High | L | — |
| 0.2 | Verify backward compatibility (existing tests pass identically) | High | S | 0.1 |
| 1.1 | Add MediaReference type | High | S | 0.1 |
| 1.2 | Extract MediaReferences from Image events | High | M | 0.1, 1.1 |
| 2.1 | Add media copy to acli | High | M | 1.1 |
| 2.2 | Detect filename collisions | High | S | 2.1 |
| 2.3 | Integrate into sync workflow | High | M | 1.2, 2.1, 2.2 |
| 3.1 | anki-wrapper media API | Low | M | — (deferred) |

---

## Open Questions (Remaining)

1. **Subdirectory organization in Art.xls**: Does the XLS have info about which subdirectory each image is in?
   - Need to investigate the XLS structure to determine this.
