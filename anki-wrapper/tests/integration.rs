#![cfg(feature = "real-anki")]

use anki_wrapper::DefaultAnkiCollection;

#[test]
#[ignore]
fn opens_real_collection_placeholder() {
    // Placeholder integration test; will be implemented with real Anki collection path.
    let _ = DefaultAnkiCollection::new();
}

