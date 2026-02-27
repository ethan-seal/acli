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

## Phase 1: Parser Enhancement

### Task 1.1: Add MediaReference type to doc-parser

Add a new type to track image references discovered during parsing.

**File:** `doc-parser/src/types.rs`

```rust
/// Reference to a media file discovered during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaReference {
    /// Path to source file, relative to the document.
    pub source_path: String,
    /// Filename to use in Anki (normalized).
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

**Tests:**
- Unit test for MediaReference construction
- Test that ParsedDocument::default() has empty media vec

### Task 1.2: Implement image syntax detection

Detect `![alt](path)` patterns in card content and extract the path.

**File:** `doc-parser/src/parser.rs`

Add a helper function:
```rust
/// Extract image references from content, returning (transformed_content, media_refs).
/// Replaces ![alt](path) with <img src="filename" alt="alt">
fn extract_images(content: &str) -> (String, Vec<MediaReference>) {
    // Use regex or simple parsing
    // Pattern: ![optional alt text](path/to/file.jpg)
}
```

**Tests:**
- `![](image.jpg)` -> `<img src="image.jpg">`
- `![Alt Text](dir/image.png)` -> `<img src="image.png" alt="Alt Text">`
- Multiple images in one line
- No images (passthrough)
- Mixed text and images

### Task 1.3: Integrate image extraction into parser

Modify `MarkdownParser::parse()` to call `extract_images()` on card content.

**Changes:**
- Before creating a Card, run both LHS and RHS through `extract_images()`
- Collect MediaReferences in a Vec<MediaReference> on ParsedDocument
- Deduplicate media references (same source path only needs to be copied once)

**Tests:**
- Parse `- ![](foo.jpg) -> answer` produces card with `<img src="foo.jpg">` front
- Parse `- question -> ![](result.jpg)` produces card with image in answer
- Bidirectional cards with images
- Nested context with images
- Multiple cards referencing same image (dedup)

---

## Phase 2: CLI Media Handling

### Task 2.1: Add media copy functionality to acli

During sync, copy referenced media files to Anki's collection.media folder.

**File:** `src/main.rs` or new `src/media.rs`

```rust
/// Copy media files from source locations to Anki's media folder.
/// Returns mapping of source paths to actual Anki filenames.
fn copy_media_to_anki(
    document_path: &Path,
    media_refs: &[MediaReference],
    anki_media_folder: &Path,
) -> Result<HashMap<String, String>, Error> {
    // For each reference:
    // 1. Resolve source path relative to document
    // 2. Read file bytes
    // 3. Copy to anki_media_folder with normalized name
    // 4. Return mapping
}
```

**Tests:**
- Copies file to destination
- Handles missing source files gracefully (error with good message)
- Handles filename collisions (Anki's SHA1 suffix pattern)
- Relative path resolution

### Task 2.2: Integrate media handling into sync workflow

Modify the sync command to:
1. Collect all MediaReferences from parsed documents
2. Copy media files before creating/updating cards
3. Report media copy results

**Tests:**
- Full sync with image cards (integration test)
- Sync skips already-present media files
- Clear error message for missing images

---

## Phase 3: anki-wrapper Media API (Optional)

If we need more control, expose Anki's MediaManager:

### Task 3.1: Add media operations to AnkiCollection trait

```rust
trait AnkiCollection {
    // Existing methods...
    
    /// Add a media file to the collection.
    /// Returns the actual filename used (may differ due to normalization).
    fn add_media_file(&mut self, name: &str, data: &[u8]) -> Result<String, Error>;
    
    /// Check if a media file exists.
    fn media_file_exists(&self, name: &str) -> Result<bool, Error>;
}
```

This phase is optional - direct file copying to collection.media may be sufficient for MVP.

---

## Testing Strategy

### Unit Tests (doc-parser)
- Image syntax parsing
- MediaReference extraction
- HTML tag generation
- Edge cases (no images, multiple images, nested contexts)

### Integration Tests (acli)
- Full sync with image cards
- Media file copying
- Error handling for missing files

### E2E Demo
- Add an Art cards example to e2e demo
- Verify images appear correctly in Anki

---

## Migration: Converting Art.xls

Script to convert the existing Art.xls to Markdown:

```python
#!/usr/bin/env python3
import pandas as pd
from pathlib import Path

# Read XLS
df = pd.read_excel('Art.xls', header=None)

# Group by directory (inferred from first column)
for idx, row in df.iterrows():
    img_path = row[0]  # e.g., "pollice.jpg"
    info = row[1]      # e.g., "Pollice Verso\nJean-Léon Gérôme"
    
    title, artist = info.split('\n', 1)
    
    # Determine subdirectory from image location
    # Output Markdown format
    print(f"- ![]({img_path}) -> {title} by {artist}")
```

---

## Task Summary

| ID | Task | Priority | Est. Size |
|----|------|----------|-----------|
| 1.1 | Add MediaReference type | High | S |
| 1.2 | Implement image syntax detection | High | M |
| 1.3 | Integrate into parser | High | M |
| 2.1 | Add media copy to acli | High | M |
| 2.2 | Integrate into sync workflow | High | M |
| 3.1 | anki-wrapper media API | Low | M |

---

## Open Questions

1. **Path resolution**: Should image paths be relative to the document file or to a configured root?
   - Recommendation: Relative to document file (most intuitive)

2. **Missing images**: Fail the whole sync or warn and continue?
   - Recommendation: Warn and continue (more user-friendly)

3. **Image on both sides**: How to handle `![](a.jpg) <-> ![](b.jpg)`?
   - Should work naturally - both sides get converted to HTML

4. **Subdirectory organization**: Does the XLS have info about which subdirectory each image is in?
   - Need to investigate the XLS structure more to determine this
