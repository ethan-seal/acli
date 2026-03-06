# acli

A CLI that syncs Markdown files to Anki flashcards. Documents are the source of truth — acli handles creating, updating, and deleting cards in your collection.

## Quickstart

### Build

```bash
git clone --recurse-submodules <repo-url>
cd acli

# Preview/validate only (no Anki dependency):
cargo build

# Full Anki integration:
cargo build -p acli --features real-anki --manifest-path acli/Cargo.toml
```

### Install

```bash
cargo install --path acli                          # without Anki backend
cargo install --path acli --features real-anki     # with Anki backend
```

### Usage

```bash
# Check that your markdown parses correctly
acli validate --source ./notes

# Preview cards without touching Anki
acli preview --source ./notes --deck "My Deck"

# Sync to Anki (requires real-anki feature, close Anki first)
acli sync --source ./notes --deck "My Deck"

# Point to a specific collection
acli sync --source ./notes --deck "My Deck" --collection ~/path/to/collection.anki2
```

## Writing Cards

Use arrow syntax in Markdown files. See [docs/writing-cards.md](docs/writing-cards.md) for the full reference.

```markdown
- hello -> world                   # basic card
- hello <-> hola                   # bidirectional card
- Spanish
    - Greetings
        - goodbye <-> adios        # nested context becomes part of the question

# Hydrogen
- symbol -> H                      # attribute card (heading = subject)
- atomic number -> 1
```

## Project Structure

| Crate | Purpose |
|-------|---------|
| `acli` | CLI binary — ties everything together |
| `doc-parser` | Parses Markdown into card structs |
| `update-planner-parser` | Diffs current vs. previous cards into add/update/delete ops |
| `anki-wrapper` | Thin wrapper around Anki's Rust library (with a fake for testing) |
