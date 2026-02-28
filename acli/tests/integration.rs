//! End-to-end integration tests for the full sync pipeline.
//!
//! These tests exercise the complete flow:
//! 1. Parse markdown files from disk
//! 2. Generate sync plan via update-planner
//! 3. Execute against a test Anki collection
//! 4. Verify cards were created/updated/deleted correctly

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use acli::adapter::AnkiCollectionAdapter;
use acli::{AnkiCli, SyncConfig};
use anki_wrapper::FakeAnkiCollection;
use doc_parser::{DocumentParser, MarkdownParser};
use update_planner_parser::{
    DefaultExecutor, DocumentSet, PlanExecutor, SimplePlanner, UpdatePlanner,
};

/// Helper to create a temp directory with markdown files.
fn setup_temp_dir_with_files(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    for (name, content) in files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dirs");
        }
        fs::write(&path, content).expect("failed to write test file");
    }
    dir
}

/// Run the full sync pipeline against a FakeAnkiCollection, returning the adapter.
fn run_sync_pipeline(
    source_dir: &Path,
    deck_name: &str,
) -> AnkiCollectionAdapter<FakeAnkiCollection> {
    let cli = AnkiCli::new();

    // Step 1: Discover and parse markdown files
    let files = cli
        .discover_files(&[source_dir.to_path_buf()], true)
        .expect("failed to discover files");
    assert!(!files.is_empty(), "no markdown files discovered");

    let parser = MarkdownParser::new();
    let mut all_cards = Vec::new();
    let mut source_files = Vec::new();

    for path in &files {
        let content = fs::read_to_string(path).expect("failed to read file");
        let parsed = parser.parse(&content).expect("failed to parse markdown");
        all_cards.extend(parsed.cards);
        source_files.push(path.display().to_string());
    }

    // Step 2: Convert to DocumentSet and generate sync plan
    let document_set = DocumentSet {
        cards: all_cards
            .iter()
            .map(|c| {
                let card_type = match c.card_type {
                    doc_parser::CardType::Basic => update_planner_parser::CardType::Basic,
                    doc_parser::CardType::Bidirectional => {
                        update_planner_parser::CardType::Bidirectional
                    }
                };
                update_planner_parser::Card {
                    card_type,
                    fields: c.fields.clone(),
                }
            })
            .collect(),
        source_files,
    };

    let planner = SimplePlanner;
    let plan = planner
        .plan_fresh_sync(&document_set, deck_name)
        .expect("failed to create sync plan");

    // Step 3: Execute plan against FakeAnkiCollection
    let executor = DefaultExecutor;
    let mut adapter = AnkiCollectionAdapter::fake();
    executor
        .execute_plan(&plan, &mut adapter)
        .expect("failed to execute plan");

    adapter
}

// =============================================================================
// Test: Basic card creation
// =============================================================================

#[test]
fn test_sync_creates_cards_from_single_file() {
    let content = r#"- What is Rust? -> A systems programming language
- Who created Rust? -> Graydon Hoare
"#;

    let temp_dir = setup_temp_dir_with_files(&[("flashcards.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Test::Deck");

    // Verify cards were created
    let deck = adapter
        .inner()
        .decks
        .get("Test::Deck")
        .expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 cards in deck");

    // Verify card content
    let fields: Vec<_> = deck.iter().map(|c| c.fields.clone()).collect();
    assert!(
        fields.iter().any(|f| f[0].contains("What is Rust?")),
        "first card not found"
    );
    assert!(
        fields.iter().any(|f| f[0].contains("Who created Rust?")),
        "second card not found"
    );
}

#[test]
fn test_sync_creates_cards_from_multiple_files() {
    let file1 = r#"- Capital of France? -> Paris
"#;
    let file2 = r#"- Capital of Germany? -> Berlin
- Capital of Spain? -> Madrid
"#;

    let temp_dir = setup_temp_dir_with_files(&[("geo1.md", file1), ("geo2.md", file2)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Geography");

    let deck = adapter
        .inner()
        .decks
        .get("Geography")
        .expect("deck not found");
    assert_eq!(deck.len(), 3, "expected 3 cards in deck");
}

#[test]
fn test_sync_handles_nested_directories() {
    let file1 = r#"- Math fact -> 2+2=4
"#;
    let file2 = r#"- Science fact -> Water is H2O
"#;

    let temp_dir =
        setup_temp_dir_with_files(&[("math/basics.md", file1), ("science/chemistry.md", file2)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Study");

    let deck = adapter.inner().decks.get("Study").expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 cards from nested dirs");
}

// =============================================================================
// Test: Bidirectional cards
// =============================================================================

#[test]
fn test_sync_creates_bidirectional_cards() {
    let content = r#"- Dog <-> Perro
- Cat <-> Gato
"#;

    let temp_dir = setup_temp_dir_with_files(&[("spanish.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Spanish");

    let deck = adapter
        .inner()
        .decks
        .get("Spanish")
        .expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 bidirectional cards");

    // Verify card types
    for card in deck {
        assert_eq!(
            card.card_type,
            anki_wrapper::CardType::BasicReversed,
            "expected bidirectional card type"
        );
    }
}

#[test]
fn test_sync_handles_mixed_card_types() {
    let content = r#"- Basic card -> Answer
- Bidirectional <-> Both ways
"#;

    let temp_dir = setup_temp_dir_with_files(&[("mixed.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Mixed");

    let deck = adapter.inner().decks.get("Mixed").expect("deck not found");
    assert_eq!(deck.len(), 2);

    let basic_count = deck
        .iter()
        .filter(|c| c.card_type == anki_wrapper::CardType::Basic)
        .count();
    let bidi_count = deck
        .iter()
        .filter(|c| c.card_type == anki_wrapper::CardType::BasicReversed)
        .count();

    assert_eq!(basic_count, 1, "expected 1 basic card");
    assert_eq!(bidi_count, 1, "expected 1 bidirectional card");
}

// =============================================================================
// Test: Re-sync scenarios (simulating updates)
// =============================================================================

#[test]
fn test_resync_with_added_cards() {
    // First sync with 1 card
    let content_v1 = r#"- Card 1 -> Answer 1
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content_v1)]);
    let adapter1 = run_sync_pipeline(temp_dir.path(), "Deck");
    assert_eq!(adapter1.inner().decks["Deck"].len(), 1);

    // Update file with 2 more cards and re-sync
    let content_v2 = r#"- Card 1 -> Answer 1
- Card 2 -> Answer 2
- Card 3 -> Answer 3
"#;
    fs::write(temp_dir.path().join("cards.md"), content_v2).expect("failed to update file");

    let adapter2 = run_sync_pipeline(temp_dir.path(), "Deck");
    assert_eq!(
        adapter2.inner().decks["Deck"].len(),
        3,
        "expected 3 cards after adding more"
    );
}

#[test]
fn test_resync_with_removed_cards() {
    // First sync with 3 cards
    let content_v1 = r#"- Card 1 -> Answer 1
- Card 2 -> Answer 2
- Card 3 -> Answer 3
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content_v1)]);
    let adapter1 = run_sync_pipeline(temp_dir.path(), "Deck");
    assert_eq!(adapter1.inner().decks["Deck"].len(), 3);

    // Update file with only 1 card and re-sync
    let content_v2 = r#"- Card 1 -> Answer 1
"#;
    fs::write(temp_dir.path().join("cards.md"), content_v2).expect("failed to update file");

    let adapter2 = run_sync_pipeline(temp_dir.path(), "Deck");
    assert_eq!(
        adapter2.inner().decks["Deck"].len(),
        1,
        "expected 1 card after removing some"
    );
}

#[test]
fn test_resync_with_modified_cards() {
    // First sync
    let content_v1 = r#"- Question -> Old Answer
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content_v1)]);
    let adapter1 = run_sync_pipeline(temp_dir.path(), "Deck");
    let old_answer = &adapter1.inner().decks["Deck"][0].fields[1];
    assert_eq!(old_answer, "Old Answer");

    // Update file with modified answer and re-sync
    let content_v2 = r#"- Question -> New Answer
"#;
    fs::write(temp_dir.path().join("cards.md"), content_v2).expect("failed to update file");

    let adapter2 = run_sync_pipeline(temp_dir.path(), "Deck");
    let new_answer = &adapter2.inner().decks["Deck"][0].fields[1];
    assert_eq!(new_answer, "New Answer", "answer should be updated");
}

// =============================================================================
// Test: Nested list context
// =============================================================================

#[test]
fn test_sync_preserves_nested_context() {
    let content = r#"- Programming
    - Languages
        - Rust -> Systems language
        - Python -> Scripting language
"#;

    let temp_dir = setup_temp_dir_with_files(&[("nested.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Programming");

    let deck = adapter
        .inner()
        .decks
        .get("Programming")
        .expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 cards from nested list");

    // Verify context is preserved in questions
    let rust_card = deck.iter().find(|c| c.fields[1] == "Systems language");
    assert!(rust_card.is_some(), "Rust card not found");

    let question = &rust_card.unwrap().fields[0];
    assert!(
        question.contains("Programming"),
        "question should contain parent context"
    );
    assert!(
        question.contains("Languages"),
        "question should contain parent context"
    );
}

// =============================================================================
// Test: Edge cases
// =============================================================================

#[test]
fn test_sync_with_special_characters_in_content() {
    let content = r#"- What is 2 + 2? -> 4
- HTML tag <div> -> Defines a division
- SQL: SELECT * FROM users -> Retrieves all users
"#;

    let temp_dir = setup_temp_dir_with_files(&[("special.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Special");

    let deck = adapter
        .inner()
        .decks
        .get("Special")
        .expect("deck not found");
    assert_eq!(deck.len(), 3, "expected 3 cards with special characters");
}

#[test]
fn test_sync_with_unicode_content() {
    let content = r#"- こんにちは -> Hello in Japanese
- Привет -> Hello in Russian
- 你好 -> Hello in Chinese
"#;

    let temp_dir = setup_temp_dir_with_files(&[("unicode.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Unicode");

    let deck = adapter
        .inner()
        .decks
        .get("Unicode")
        .expect("deck not found");
    assert_eq!(deck.len(), 3, "expected 3 unicode cards");

    // Verify unicode preserved
    let japanese_card = deck.iter().find(|c| c.fields[1] == "Hello in Japanese");
    assert!(japanese_card.is_some());
    assert!(japanese_card.unwrap().fields[0].contains("こんにちは"));
}

#[test]
fn test_sync_ignores_non_markdown_files() {
    let md_content = r#"- Real card -> Real answer
"#;
    let txt_content = "This is not a markdown file\nFake -> Card";

    let temp_dir = setup_temp_dir_with_files(&[
        ("cards.md", md_content),
        ("notes.txt", txt_content),
        ("data.json", "{}"),
    ]);
    let adapter = run_sync_pipeline(temp_dir.path(), "OnlyMd");

    let deck = adapter.inner().decks.get("OnlyMd").expect("deck not found");
    assert_eq!(deck.len(), 1, "should only have card from .md file");
}

// =============================================================================
// Test: Deck naming
// =============================================================================

#[test]
fn test_sync_creates_hierarchical_deck() {
    let content = r#"- Card -> Answer
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Parent::Child::Grandchild");

    assert!(
        adapter
            .inner()
            .decks
            .contains_key("Parent::Child::Grandchild"),
        "hierarchical deck should be created"
    );
}

// =============================================================================
// Test: High-level SyncConfig API
// =============================================================================

#[test]
fn test_sync_via_ankicli_dry_run() {
    let content = r#"- Card -> Answer
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content)]);

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![temp_dir.path().to_path_buf()],
        deck_name: "DryRun".to_string(),
        anki_collection_path: None,
        anki_media_dir: None,
        recursive: true,
        dry_run: true,
    };

    let result = cli.sync(&config).expect("dry run sync failed");
    assert!(result.dry_run, "should indicate dry run");
    assert_eq!(result.files_processed, 1);
    assert!(result.cards_synced > 0);
}

#[test]
fn test_sync_multiple_source_directories() {
    let dir1_content = r#"- Dir1 Card -> Answer 1
"#;
    let dir2_content = r#"- Dir2 Card -> Answer 2
"#;

    let temp_dir1 = setup_temp_dir_with_files(&[("cards.md", dir1_content)]);
    let temp_dir2 = setup_temp_dir_with_files(&[("cards.md", dir2_content)]);

    let cli = AnkiCli::new();

    // Discover files from both directories
    let files = cli
        .discover_files(
            &[
                temp_dir1.path().to_path_buf(),
                temp_dir2.path().to_path_buf(),
            ],
            true,
        )
        .expect("failed to discover files");

    assert_eq!(files.len(), 2, "should find files from both directories");
}

// =============================================================================
// Test: Media handling in sync workflow
// =============================================================================

#[test]
fn test_sync_with_image_cards_no_media_dir() {
    // Sync with image cards but no anki_media_dir — should succeed, cards created normally.
    let content = r#"- ![photo](photo.jpg) What is this? -> A photograph
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cards.md", content)]);

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![temp_dir.path().to_path_buf()],
        deck_name: "MediaTest".to_string(),
        anki_collection_path: None,
        anki_media_dir: None,
        recursive: true,
        dry_run: false,
    };

    let result = cli
        .sync(&config)
        .expect("sync with image cards should succeed");
    assert!(!result.dry_run);
    assert_eq!(result.files_processed, 1);
    assert_eq!(
        result.media_copied, 0,
        "no media dir configured, nothing copied"
    );
    assert_eq!(result.media_skipped, 0);
    assert_eq!(result.media_missing, 0);
}

#[test]
fn test_sync_copies_media_files() {
    // Create a markdown file referencing photo.jpg, create the file, run sync with media dir.
    let content = r#"- ![alt](photo.jpg) -> answer
"#;

    let doc_dir = tempfile::tempdir().expect("failed to create doc dir");
    fs::write(doc_dir.path().join("cards.md"), content).expect("failed to write md");
    fs::write(doc_dir.path().join("photo.jpg"), b"fake jpeg data").expect("failed to write image");

    let anki_media_dir = tempfile::tempdir().expect("failed to create media dir");

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![doc_dir.path().to_path_buf()],
        deck_name: "MediaCopy".to_string(),
        anki_collection_path: None,
        anki_media_dir: Some(anki_media_dir.path().to_path_buf()),
        recursive: true,
        dry_run: false,
    };

    let result = cli
        .sync(&config)
        .expect("sync with media dir should succeed");
    assert_eq!(result.media_copied, 1, "expected 1 file copied");
    assert_eq!(result.media_skipped, 0);
    assert_eq!(result.media_missing, 0);
    assert!(
        anki_media_dir.path().join("photo.jpg").exists(),
        "photo.jpg should have been copied to anki_media_dir"
    );
}

#[test]
fn test_sync_skips_already_present_media() {
    // Pre-populate anki_media_dir with the file — sync should skip it.
    let content = r#"- ![alt](photo.jpg) -> answer
"#;

    let doc_dir = tempfile::tempdir().expect("failed to create doc dir");
    fs::write(doc_dir.path().join("cards.md"), content).expect("failed to write md");
    fs::write(doc_dir.path().join("photo.jpg"), b"source image").expect("failed to write image");

    let anki_media_dir = tempfile::tempdir().expect("failed to create media dir");
    // Pre-populate destination
    fs::write(anki_media_dir.path().join("photo.jpg"), b"existing image")
        .expect("failed to write existing media");

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![doc_dir.path().to_path_buf()],
        deck_name: "MediaSkip".to_string(),
        anki_collection_path: None,
        anki_media_dir: Some(anki_media_dir.path().to_path_buf()),
        recursive: true,
        dry_run: false,
    };

    let result = cli
        .sync(&config)
        .expect("sync should succeed even with existing media");
    assert_eq!(result.media_skipped, 1, "expected 1 file skipped");
    assert_eq!(result.media_copied, 0, "expected 0 files copied");
    assert_eq!(result.media_missing, 0);
}

#[test]
fn test_sync_fails_on_media_collision() {
    // Two markdown files each reference a different source path that maps to the same target name.
    // source_path differs ("art/mona.jpg" vs "photos/mona.jpg") but target_name is "mona.jpg" for both.
    let file1 = r#"- ![](art/mona.jpg) -> painting1
"#;
    let file2 = r#"- ![](photos/mona.jpg) -> painting2
"#;

    let temp_dir = setup_temp_dir_with_files(&[("file1.md", file1), ("file2.md", file2)]);

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![temp_dir.path().to_path_buf()],
        deck_name: "CollisionTest".to_string(),
        anki_collection_path: None,
        anki_media_dir: None,
        recursive: true,
        dry_run: false,
    };

    let result = cli.sync(&config);
    assert!(
        result.is_err(),
        "sync should fail on media filename collision"
    );
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("mona.jpg") || err_msg.contains("collision"),
        "error message should mention the collision: {err_msg}"
    );
}
