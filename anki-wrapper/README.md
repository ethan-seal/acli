# anki-wrapper

A Rust wrapper for Anki's rslib that provides convenient access to Anki's card and note operations.

## Setup

### Prerequisites

1. Rust 1.89.0 (managed automatically via `rust-toolchain.toml`)
2. The Anki subrepo with protoc built

### Building

The anki-wrapper depends on Anki's rslib from a local subrepo at `anki/`. To build:

```bash
# Without real-anki feature (fake implementation only)
cargo build

# With real-anki feature (includes rslib)
PROTOC=anki/out/extracted/protoc/bin/protoc cargo build --features real-anki
```

### Changes Made to Anki Subrepo

To make rslib buildable standalone, we made one minimal change:

- **`anki/Cargo.toml`**: Added `io-util` feature to tokio dependency (line 133)

## Usage

### Running the Example

```bash
PROTOC=anki/out/extracted/protoc/bin/protoc cargo run --example rslib_demo --features real-anki
```

The example demonstrates:
- Creating an in-memory Anki collection
- Adding a note (automatically creates cards)
- Updating a note
- Retrieving a note
- Deleting a note

### Using rslib in Your Code

```rust
use anki::collection::CollectionBuilder;
use anki::notes::Note;
use anki::decks::DeckId;

// Create a collection
let mut col = CollectionBuilder::default().build()?;

// Get a notetype
let notetypes = col.get_all_notetypes()?;
let notetype = notetypes.first().unwrap();

// Create and add a note
let mut note = Note::new(notetype);
note.set_field(0, "Front text")?;
note.set_field(1, "Back text")?;

let deck_id = DeckId(1); // Default deck
col.add_note(&mut note, deck_id)?;

// Update a note
note.set_field(1, "Updated back text")?;
col.update_note(&mut note)?;

// Retrieve a note
let retrieved = col.storage.get_note(note.id)?;

// Delete notes
col.remove_notes(&[note.id])?;
```

## API Overview

### Key Types

- `Collection` - Main entry point for Anki operations
- `Note` - A note with fields (creates cards via templates)
- `Card` - Individual cards generated from notes
- `DeckId` - Identifier for decks
- `NotetypeId` - Identifier for note types

### Main Operations

- `col.add_note(&mut note, deck_id)` - Add a new note
- `col.update_note(&mut note)` - Update an existing note
- `col.remove_notes(&[note_ids])` - Delete notes
- `col.storage.get_note(note_id)` - Retrieve a note

## Architecture

- **Fake implementation**: A simple in-memory implementation for testing (default)
- **Real implementation**: Uses Anki's rslib directly (with `real-anki` feature)

## Development

### Testing

```bash
cargo test
cargo test --features real-anki
```

### Cleaning Up

The anki subrepo can be removed and re-cloned if needed:

```bash
rm -rf anki
git clone https://github.com/ankitects/anki.git
cd anki
./check  # Build protobuf descriptors and dependencies
cd ..
```

## License

MIT OR Apache-2.0 (same as the parent project)
