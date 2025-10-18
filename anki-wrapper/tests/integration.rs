#![cfg(feature = "real-anki")]

use std::path::PathBuf;
use tempfile::TempDir;

use anki_wrapper::{DefaultAnkiCollection, AnkiCollection, DeckConfig, Card, CardType};

#[test]
#[ignore]
fn real_backend_smoke_test_placeholder() {
    // This is a scaffold: once anki-wrapper/anki is populated, implement opening a collection
    // in a temp directory and perform basic deck + card operations.
    let tmp = TempDir::new().expect("create temp dir");
    let _col_path = PathBuf::from(tmp.path());

    let mut col = DefaultAnkiCollection::new();
    let deck = DeckConfig { name: "Test".into() };
    let card = Card { card_type: CardType::Basic, fields: vec!["Front".into(), "Back".into()] };

    // When implemented, these should not error.
    let _ = col.ensure_deck(&deck);
    let _ = col.add_card(&deck, &card);
    let _ = col.save();
}
