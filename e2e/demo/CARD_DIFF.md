# Card-Level Diffing in E2E Report

## Overview

The e2e demo report now includes **card-level diffing** that exactly matches the sync logic used by `acli`. This provides a clear view of what operations will be performed during sync.

## Implementation

The card-level diff is implemented in `card-diff.ts` and mirrors the Rust implementation in `update-planner-parser/src/types.rs`:

1. **Card Parsing**: Extracts cards from markdown using the same syntax (`->` for basic, `<->` for bidirectional)
2. **Card Identity**: Computes stable CardIds based on content (card type + fields)
3. **Diff Computation**: Uses the same algorithm as `DocumentDiff::compute()`

## Display Format

The report shows two types of diffs:

### 1. Card-Level Changes (Primary)
Shows the actual sync operations that will be performed:
- **Added**: New cards to create in Anki
- **Deleted**: Cards to remove from Anki
- **Updated**: Cards with same ID but different content (rare with content-based IDs)
- **Unchanged**: Cards that remain the same (shown for context)

Example:
```
Sync operations (matching acli logic)
Added (2 cards):
+ Good morning <-> Buenos dias [bidir]
+ Three -> Tres [basic]

Unchanged (4 cards):
  Hello <-> Hola
  Goodbye <-> Adios
  ...
```

### 2. Markdown Changes (Secondary, Collapsible)
Shows line-by-line text differences in the markdown file. This is hidden by default in a `<details>` element.

## Key Behaviors

### Content Changes = Delete + Add
When a card's content changes (e.g., "Goodbye <-> Adios" → "Goodbye <-> Adios / Hasta luego"), the diff shows:
```
Added (1 card):
+ Goodbye <-> Adios / Hasta luego [bidir]

Deleted (1 card):
- Goodbye <-> Adios [bidir]
```

This correctly reflects how content-based CardIds work: any content change creates a new ID, so the old card is deleted and a new one is added.

### Card Types
Each card shows its type:
- `[basic]` - One-way card (Front → Back)
- `[bidir]` - Bidirectional card (Front ↔ Back)

## Visual Design

- Card-level diff has a **pink border** to distinguish it from markdown diff
- Uses the same color coding as git diffs:
  - 🟢 Green (`+`) for additions
  - 🔴 Red (`-`) for deletions
  - ⚪ Gray for context/unchanged
- Card type badges help quickly identify bidirectional vs basic cards

## Benefits

1. **Accuracy**: Matches the actual sync implementation exactly
2. **Clarity**: Easy to see what operations will be performed
3. **Debugging**: Helps verify sync behavior is correct
4. **Documentation**: Serves as a visual example of how card identity works
