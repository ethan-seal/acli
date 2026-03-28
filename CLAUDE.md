## Issue Tracking

This project uses **bd (beads)** for issue tracking.
Run `bd prime` for workflow context, or install hooks (`bd hooks install`) for auto-injection.

**Quick reference:**
- `bd ready` - Find unblocked work
- `bd create "Title" --type task --priority 2` - Create issue
- `bd close <id>` - Complete work
- `bd sync` - Sync with git (run at session end)

For full workflow details: `bd prime`

## Project Structure

This is a Rust workspace with the following crates:

- **`acli`** - Main CLI binary; orchestrates parsing, syncing, and preview
- **`doc-parser`** - Parses flashcard syntax from markdown files (inline/block cards)
- **`anki-wrapper`** - Interfaces with Anki database via anki-rs backend
- **`web-preview`** - Local HTTP server for card preview (uses tiny_http + pulldown_cmark)

### Refactoring Library Crates

When refactoring library crates (doc-parser, anki-wrapper, web-preview):

- **Preserve public API** - Other crates depend on it; check imports with `grep -r "use cratename::" acli/`
- **Public surface in lib.rs** - Keep public types and re-exports in lib.rs, implementation in modules
- **Use `pub(crate)` for internals** - Items only used within the crate don't need `pub`
- **Tests move with code** - `#[cfg(test)]` modules stay in the same file as what they test
- **Verify both tests and build** - Run `cargo test` and `cargo build` from workspace root

## Coding Guidelines

### Meaningful abstractions
- When code grows complex, look for real abstractions — concepts that deserve a name and a boundary. The goal is to make the code easier to reason about, not just shorter. A function should represent a coherent idea, not an arbitrary chunk cut at a line count.

### Long functions and deep nesting are smells
- A function exceeding ~80 lines or nesting beyond ~4 levels is a signal to pause and consider whether it's doing too many things. These aren't hard limits — sometimes the clearest expression of an idea is a long function. But more often it means there's a meaningful sub-operation hiding inside that would be clearer with its own name and contract.

### Large files are smells
- A source file exceeding ~500 lines (excluding tests) suggests it may be covering too many concerns. Consider splitting into a module directory where each file has a focused responsibility. The question isn't "is it too long?" but "would a reader know where to look?"

### Duplication signals a missing abstraction
- If the same structural pattern appears 3+ times, that's a strong signal there's a concept worth naming. Extract it — but make sure the abstraction captures the *idea*, not just the syntax. Two occurrences are tolerable; three means you're maintaining the same logic in multiple places.

### Data over control flow
- When multiple branches differ only in literal values (strings, enum variants, config), prefer a table or array lookup over cascading if/else or match arms. Data-driven code is easier to extend and harder to get wrong.

### Pipeline consistency
- Functions that serve the same role in a pipeline should share a common return type. Don't make callers handle 3 different tuple shapes for the same kind of result.

### No dead code
- Don't keep fields, branches, or functions that are documented as always-empty or unreachable. Remove them. Dead code misleads readers and accumulates maintenance cost.
