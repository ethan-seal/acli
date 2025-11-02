// Example demonstrating direct rslib integration for add/update/delete operations
// Run with: PROTOC=anki/out/extracted/protoc/bin/protoc cargo run --example rslib_demo --features real-anki

#[cfg(feature = "real-anki")]
fn main() -> anyhow::Result<()> {
    use anki::collection::CollectionBuilder;
    use anki::notes::Note;
    use anki::decks::DeckId;

    // Create a temporary in-memory collection for demonstration
    println!("Creating in-memory Anki collection...");
    let mut col = CollectionBuilder::default().build()?;

    // Get the default deck ID (deck 1 is always the default deck)
    let deck_id = DeckId(1);

    // Get an existing notetype (every new collection has a "Basic" notetype)
    println!("Getting note type...");
    let notetypes = col.get_all_notetypes()?;
    let basic_notetype = notetypes.first()
        .expect("At least one notetype should exist");

    println!("Using notetype: {}", basic_notetype.name);

    // Example 1: Add a new note
    println!("\n=== Adding a new note ===");
    let mut note = Note::new(basic_notetype);
    note.set_field(0, "What is Rust?")?;
    note.set_field(1, "A systems programming language")?;

    match col.add_note(&mut note, deck_id) {
        Ok(output) => {
            println!("✓ Note added successfully!");
            println!("  Note ID: {:?}", note.id);
            println!("  Cards generated: {}", output.output);
        }
        Err(e) => println!("✗ Failed to add note: {}", e),
    }

    // Example 2: Update the note
    println!("\n=== Updating the note ===");
    note.set_field(1, "A fast, safe systems programming language")?;

    match col.update_note(&mut note) {
        Ok(_) => println!("✓ Note updated successfully!"),
        Err(e) => println!("✗ Failed to update note: {}", e),
    }

    // Example 3: Get the note back to verify
    println!("\n=== Retrieving note ===");
    match col.storage.get_note(note.id) {
        Ok(Some(retrieved_note)) => {
            println!("✓ Note retrieved:");
            println!("  Front: {}", retrieved_note.fields()[0]);
            println!("  Back: {}", retrieved_note.fields()[1]);
        }
        Ok(None) => println!("✗ Note not found"),
        Err(e) => println!("✗ Failed to retrieve note: {}", e),
    }

    // Example 4: Delete the note
    println!("\n=== Deleting the note ===");
    match col.remove_notes(&[note.id]) {
        Ok(output) => {
            println!("✓ Note deleted successfully!");
            println!("  Notes removed: {}", output.output);
        }
        Err(e) => println!("✗ Failed to delete note: {}", e),
    }

    println!("\n=== Summary ===");
    println!("Successfully demonstrated:");
    println!("  - Creating an Anki collection");
    println!("  - Adding a note (creates cards automatically)");
    println!("  - Updating a note");
    println!("  - Retrieving a note");
    println!("  - Deleting a note");

    Ok(())
}

#[cfg(not(feature = "real-anki"))]
fn main() {
    eprintln!("This example requires the 'real-anki' feature.");
    eprintln!("Run with: PROTOC=anki/out/extracted/protoc/bin/protoc cargo run --example rslib_demo --features real-anki");
    std::process::exit(1);
}
