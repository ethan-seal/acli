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
                    doc_parser::CardType::Sequence => update_planner_parser::CardType::Sequence,
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
    assert_eq!(old_answer, "<p>Old Answer</p>");

    // Update file with modified answer and re-sync
    let content_v2 = r#"- Question -> New Answer
"#;
    fs::write(temp_dir.path().join("cards.md"), content_v2).expect("failed to update file");

    let adapter2 = run_sync_pipeline(temp_dir.path(), "Deck");
    let new_answer = &adapter2.inner().decks["Deck"][0].fields[1];
    assert_eq!(new_answer, "<p>New Answer</p>", "answer should be updated");
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
    let rust_card = deck
        .iter()
        .find(|c| c.fields[1].contains("Systems language"));
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
    let japanese_card = deck
        .iter()
        .find(|c| c.fields[1].contains("Hello in Japanese"));
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

// =============================================================================
// Test: Attribute cards (heading + key -> value)
// =============================================================================

#[test]
fn test_sync_attribute_cards_with_heading() {
    let content = "# Hydrogen\n- symbol -> H\n- atomic number -> 1\n- phase -> gas\n";

    let temp_dir = setup_temp_dir_with_files(&[("elements.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Chemistry");

    let deck = adapter
        .inner()
        .decks
        .get("Chemistry")
        .expect("deck not found");
    assert_eq!(deck.len(), 3, "expected 3 attribute cards");

    // Each card front should contain the heading "Hydrogen" and the arrow "-> ?"
    // (HTML-encoded as "-&gt; ?" by pulldown-cmark).
    for card in deck.iter() {
        assert!(
            card.fields[0].contains("Hydrogen"),
            "card front should contain heading: {:?}",
            card.fields[0]
        );
        assert!(
            card.fields[0].contains("-&gt; ?"),
            "card front should contain -> ?: {:?}",
            card.fields[0]
        );
    }

    // Verify individual card answers
    let values: Vec<&str> = deck.iter().map(|c| c.fields[1].as_str()).collect();
    assert!(values.iter().any(|v| v.contains('H')), "missing H card");
    assert!(
        values.iter().any(|v| v.contains('1')),
        "missing atomic number card"
    );
    assert!(
        values.iter().any(|v| v.contains("gas")),
        "missing phase card"
    );
}

#[test]
fn test_sync_attribute_cards_no_heading_is_plain_basic() {
    // Without a heading, -> items are plain basic cards.
    let content = "- symbol -> H\n";

    let temp_dir = setup_temp_dir_with_files(&[("plain.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Plain");

    let deck = adapter.inner().decks.get("Plain").expect("deck not found");
    assert_eq!(deck.len(), 1);

    // Front should contain "-> ?" (HTML-encoded as "-&gt; ?" by pulldown-cmark).
    assert!(
        deck[0].fields[0].contains("-&gt; ?"),
        "plain card front should contain -> ? (html-encoded): {:?}",
        deck[0].fields[0]
    );
}

#[test]
fn test_sync_attribute_cards_heading_resets_between_sections() {
    let content = "# Hydrogen\n- symbol -> H\n\n# Oxygen\n- symbol -> O\n";

    let temp_dir = setup_temp_dir_with_files(&[("elements.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Elements");

    let deck = adapter
        .inner()
        .decks
        .get("Elements")
        .expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 attribute cards");

    let hydrogen_card = deck.iter().find(|c| c.fields[1].contains('H'));
    let oxygen_card = deck.iter().find(|c| c.fields[1].contains('O'));

    assert!(hydrogen_card.is_some(), "Hydrogen card not found");
    assert!(oxygen_card.is_some(), "Oxygen card not found");

    assert!(
        hydrogen_card.unwrap().fields[0].contains("Hydrogen"),
        "H card should reference Hydrogen heading"
    );
    assert!(
        oxygen_card.unwrap().fields[0].contains("Oxygen"),
        "O card should reference Oxygen heading"
    );
}

// =============================================================================
// Test: Pipe table cards
// =============================================================================

#[test]
fn test_sync_pipe_table_bidirectional() {
    // A pipe table with a <-> column should produce forward + reverse cards.
    let content = "subject | conjugation <->\nyo | soy\ntú | eres\n";

    let temp_dir = setup_temp_dir_with_files(&[("verbs.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Spanish");

    let deck = adapter
        .inner()
        .decks
        .get("Spanish")
        .expect("deck not found");

    // 2 rows × 2 directions = 4 cards
    assert_eq!(deck.len(), 4, "expected 4 pipe-table cards");

    // Every card should be Basic type (forward/reverse are separate Basic cards).
    for card in deck.iter() {
        assert_eq!(
            card.card_type,
            anki_wrapper::CardType::Basic,
            "pipe-table cards should be Basic note type"
        );
    }

    // Verify context appears in all fronts (HTML-encoded: → becomes &rarr; or raw Unicode).
    for card in deck.iter() {
        assert!(
            card.fields[0].contains("subject") && card.fields[0].contains("conjugation"),
            "context 'subject → conjugation' should appear in front: {:?}",
            card.fields[0]
        );
    }

    // Forward card for "yo": back should be "soy"
    let fwd_yo = deck
        .iter()
        .find(|c| c.fields[1].contains("soy") && c.fields[0].contains("yo"));
    assert!(fwd_yo.is_some(), "forward card for yo→soy not found");

    // Reverse card for "soy": back should be "yo"
    let rev_soy = deck
        .iter()
        .find(|c| c.fields[1].contains("yo") && c.fields[0].contains("soy"));
    assert!(rev_soy.is_some(), "reverse card for soy→yo not found");
}

#[test]
fn test_sync_pipe_table_forward_only() {
    let content = "term | definition ->\nhello | a greeting\nbye | a farewell\n";

    let temp_dir = setup_temp_dir_with_files(&[("terms.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Terms");

    let deck = adapter.inner().decks.get("Terms").expect("deck not found");

    // Forward-only: 2 rows × 1 direction = 2 cards
    assert_eq!(deck.len(), 2, "expected 2 forward-only cards");

    // All backs should be definitions (not row values).
    let backs: Vec<&str> = deck.iter().map(|c| c.fields[1].as_str()).collect();
    assert!(backs.iter().any(|b| b.contains("a greeting")));
    assert!(backs.iter().any(|b| b.contains("a farewell")));
}

#[test]
fn test_sync_pipe_table_empty_cell_skipped() {
    // Rows with missing cell values should not generate cards for that cell.
    let content = "subject | value <->\nrow1 | val1\nrow2 |\n";

    let temp_dir = setup_temp_dir_with_files(&[("table.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Table");

    let deck = adapter.inner().decks.get("Table").expect("deck not found");

    // Only row1 produces cards (2); row2 is skipped.
    assert_eq!(deck.len(), 2, "expected 2 cards (empty cell row skipped)");
}

#[test]
fn test_sync_pipe_table_mixed_with_basic_cards() {
    let content = "subject | conjugation <->\nyo | soy\n\n- Capital of France? -> Paris\n";

    let temp_dir = setup_temp_dir_with_files(&[("mixed.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Mixed");

    let deck = adapter.inner().decks.get("Mixed").expect("deck not found");

    // 2 pipe-table cards + 1 basic card = 3 total
    assert_eq!(deck.len(), 3, "expected 2 pipe-table + 1 basic card");
}

// =============================================================================
// Test: Ordered sequence cards (=> prefix)
// =============================================================================

#[test]
fn test_sync_sequence_cards_are_created() {
    let content = "Troubleshoot Wi-Fi connection\n=> check Wi-Fi is enabled\n=> restart device\n=> forget network and reconnect\n";

    let temp_dir = setup_temp_dir_with_files(&[("wifi.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Troubleshooting");

    let deck = adapter
        .inner()
        .decks
        .get("Troubleshooting")
        .expect("deck not found");

    assert_eq!(deck.len(), 3, "expected 3 sequence cards in deck");

    // All sequence cards are stored as Basic notes in Anki.
    for card in deck.iter() {
        assert_eq!(
            card.card_type,
            anki_wrapper::CardType::Basic,
            "sequence cards should use Basic note type"
        );
    }

    // Verify front fields contain the label and correct prompt.
    let fields: Vec<_> = deck.iter().map(|c| c.fields.clone()).collect();

    // Card 1: "Troubleshoot Wi-Fi connection\nFirst:" (rendered as HTML)
    let first_front = fields.iter().find(|f| f[0].contains("First:"));
    assert!(
        first_front.is_some(),
        "first sequence card should contain 'First:' in front"
    );

    // Card 2 / 3: should contain "After:"
    let after_count = fields.iter().filter(|f| f[0].contains("After:")).count();
    assert_eq!(
        after_count, 2,
        "cards 2 and 3 should contain 'After:' in front"
    );
}

#[test]
fn test_sync_sequence_mixed_with_basic_cards() {
    let content =
        "Boot steps\n=> power off\n=> wait\n=> power on\n\n- Capital of France? -> Paris\n";

    let temp_dir = setup_temp_dir_with_files(&[("mixed.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Mixed");

    let deck = adapter.inner().decks.get("Mixed").expect("deck not found");

    // 3 sequence cards + 1 basic card
    assert_eq!(
        deck.len(),
        4,
        "expected 4 cards total (3 sequence + 1 basic)"
    );
}

#[test]
fn test_sync_sequence_front_contains_label_and_step() {
    let content = "Deploy procedure\n=> build the project\n=> run tests\n";

    let temp_dir = setup_temp_dir_with_files(&[("deploy.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Deploy");

    let deck = adapter.inner().decks.get("Deploy").expect("deck not found");
    assert_eq!(deck.len(), 2);

    // Back fields should be the step texts (possibly HTML-encoded).
    let backs: Vec<&str> = deck.iter().map(|c| c.fields[1].as_str()).collect();
    assert!(
        backs.iter().any(|b| b.contains("build the project")),
        "back should contain first step text; got: {backs:?}"
    );
    assert!(
        backs.iter().any(|b| b.contains("run tests")),
        "back should contain second step text; got: {backs:?}"
    );
}

// =============================================================================
// Test: Incremental sync — reviews survive re-sync
// =============================================================================

/// Helper: parse markdown content and return the cards.
fn parse_cards(content: &str) -> Vec<doc_parser::Card> {
    let parser = MarkdownParser::new();
    parser.parse(content).expect("failed to parse").cards
}

/// Helper: run incremental sync against a shared adapter, returning counts.
fn run_incremental_sync(
    content: &str,
    deck_name: &str,
    adapter: &mut AnkiCollectionAdapter<FakeAnkiCollection>,
) -> acli::SyncCounts {
    let cards = parse_cards(content);
    acli::sync_incremental(&cards, deck_name, adapter).expect("incremental sync failed")
}

#[test]
fn test_incremental_sync_unchanged_cards_keep_reviews() {
    use anki_wrapper::{AnkiCollection, ReviewRating};

    let content = r#"- What is Rust? -> A systems programming language
- Who created Rust? -> Graydon Hoare
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync: adds both cards
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 2);
    assert_eq!(counts.deleted, 0);
    assert_eq!(counts.unchanged, 0);
    assert_eq!(adapter.inner().decks["Deck"].len(), 2);

    // Simulate reviews on both cards
    let card_ids: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();
    for &cid in &card_ids {
        adapter
            .inner_mut()
            .record_review(cid, ReviewRating::Good)
            .unwrap();
    }

    // Verify reviews exist
    for &cid in &card_ids {
        let reviews = adapter.inner_mut().get_reviews(cid).unwrap();
        assert_eq!(reviews.len(), 1, "expected 1 review before re-sync");
    }

    // Re-sync with identical content
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 0, "no cards should be added");
    assert_eq!(counts.deleted, 0, "no cards should be deleted");
    assert_eq!(counts.unchanged, 2, "both cards should be unchanged");

    // Verify cards are still there with same IDs
    let card_ids_after: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();
    assert_eq!(card_ids, card_ids_after, "card IDs should be preserved");

    // Verify reviews are still there
    for &cid in &card_ids {
        let reviews = adapter.inner_mut().get_reviews(cid).unwrap();
        assert_eq!(reviews.len(), 1, "review should survive re-sync");
        assert_eq!(reviews[0].rating, ReviewRating::Good);
    }
}

#[test]
fn test_incremental_sync_add_new_cards_preserves_existing() {
    use anki_wrapper::{AnkiCollection, ReviewRating};

    let content_v1 = r#"- Card 1 -> Answer 1
"#;
    let content_v2 = r#"- Card 1 -> Answer 1
- Card 2 -> Answer 2
- Card 3 -> Answer 3
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync: 1 card
    let counts = run_incremental_sync(content_v1, "Deck", &mut adapter);
    assert_eq!(counts.added, 1);

    // Review the card
    let card1_id = adapter.inner().decks["Deck"][0].id;
    adapter
        .inner_mut()
        .record_review(card1_id, ReviewRating::Easy)
        .unwrap();

    // Second sync: adds 2 more cards
    let counts = run_incremental_sync(content_v2, "Deck", &mut adapter);
    assert_eq!(counts.added, 2, "2 new cards should be added");
    assert_eq!(counts.deleted, 0, "no cards should be deleted");
    assert_eq!(counts.unchanged, 1, "original card should be unchanged");
    assert_eq!(adapter.inner().decks["Deck"].len(), 3);

    // Original card's review is preserved
    let reviews = adapter.inner_mut().get_reviews(card1_id).unwrap();
    assert_eq!(reviews.len(), 1, "review on original card should survive");
}

#[test]
fn test_incremental_sync_remove_cards_preserves_remaining() {
    use anki_wrapper::{AnkiCollection, ReviewRating};

    let content_v1 = r#"- Card 1 -> Answer 1
- Card 2 -> Answer 2
- Card 3 -> Answer 3
"#;
    let content_v2 = r#"- Card 1 -> Answer 1
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync: 3 cards
    run_incremental_sync(content_v1, "Deck", &mut adapter);
    assert_eq!(adapter.inner().decks["Deck"].len(), 3);

    // Review all cards
    let card_ids: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();
    for &cid in &card_ids {
        adapter
            .inner_mut()
            .record_review(cid, ReviewRating::Good)
            .unwrap();
    }

    // Second sync: remove 2 cards
    let counts = run_incremental_sync(content_v2, "Deck", &mut adapter);
    assert_eq!(counts.added, 0);
    assert_eq!(counts.deleted, 2, "2 cards should be deleted");
    assert_eq!(counts.unchanged, 1, "1 card should remain unchanged");
    assert_eq!(adapter.inner().decks["Deck"].len(), 1);

    // The remaining card keeps its review
    let remaining_id = adapter.inner().decks["Deck"][0].id;
    let reviews = adapter.inner_mut().get_reviews(remaining_id).unwrap();
    assert_eq!(reviews.len(), 1, "remaining card should keep its review");
}

#[test]
fn test_incremental_sync_edited_card_loses_review() {
    // Editing a card's content changes its identity, so the old card is deleted
    // and a new one is added. The review on the old card is lost.
    // This documents the known limitation.
    use anki_wrapper::{AnkiCollection, ReviewRating};

    let content_v1 = r#"- Question -> Old Answer
"#;
    let content_v2 = r#"- Question -> New Answer
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync
    run_incremental_sync(content_v1, "Deck", &mut adapter);
    let old_card_id = adapter.inner().decks["Deck"][0].id;

    // Review the card
    adapter
        .inner_mut()
        .record_review(old_card_id, ReviewRating::Good)
        .unwrap();

    // Edit and re-sync — card content changed, so it's a new card
    let counts = run_incremental_sync(content_v2, "Deck", &mut adapter);
    assert_eq!(counts.added, 1, "new version of card should be added");
    assert_eq!(counts.deleted, 1, "old version of card should be deleted");
    assert_eq!(counts.unchanged, 0, "no unchanged cards");
    assert_eq!(adapter.inner().decks["Deck"].len(), 1);

    // The new card has a different ID — review is lost
    let new_card_id = adapter.inner().decks["Deck"][0].id;
    assert_ne!(old_card_id, new_card_id, "edited card should get new ID");

    let new_reviews = adapter.inner_mut().get_reviews(new_card_id).unwrap();
    assert!(
        new_reviews.is_empty(),
        "edited card should have no reviews (review history is lost on edit)"
    );

    // Old card's review is orphaned in the reviews map
    let old_reviews = adapter.inner_mut().get_reviews(old_card_id).unwrap();
    assert_eq!(
        old_reviews.len(),
        1,
        "old card's review still exists but card is gone"
    );
}

#[test]
fn test_incremental_sync_multiple_resyncs_are_stable() {
    // Syncing the same content 3 times should be a no-op after the first.
    let content = r#"- A -> 1
- B -> 2
- C -> 3
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 3);
    let card_ids_v1: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();

    // Second sync — no changes
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 0);
    assert_eq!(counts.deleted, 0);
    assert_eq!(counts.unchanged, 3);
    let card_ids_v2: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();
    assert_eq!(card_ids_v1, card_ids_v2);

    // Third sync — still no changes
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 0);
    assert_eq!(counts.deleted, 0);
    assert_eq!(counts.unchanged, 3);
    let card_ids_v3: Vec<_> = adapter.inner().decks["Deck"].iter().map(|c| c.id).collect();
    assert_eq!(card_ids_v1, card_ids_v3);
}

#[test]
fn test_incremental_sync_duplicate_cards_handled() {
    // If the markdown has two identical cards, both should be created on first sync,
    // and both should be matched on re-sync.
    let content = r#"- Same Question -> Same Answer
- Same Question -> Same Answer
"#;

    let mut adapter = AnkiCollectionAdapter::fake();

    // First sync: both duplicates are added
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 2);
    assert_eq!(adapter.inner().decks["Deck"].len(), 2);

    // Re-sync: both should match as unchanged
    let counts = run_incremental_sync(content, "Deck", &mut adapter);
    assert_eq!(counts.added, 0);
    assert_eq!(counts.deleted, 0);
    assert_eq!(counts.unchanged, 2);
}

// ── Validate command tests ───────────────────────────────────────────────────

#[test]
fn test_validate_skips_files_without_cards() {
    let dir = setup_temp_dir_with_files(&[
        ("cards.md", "- Question -> Answer\n"),
        ("readme.md", "# Just a readme\n\nNo cards here.\n"),
    ]);

    let cli = AnkiCli::new();
    let config = acli::ValidationConfig {
        source_dirs: vec![dir.path().to_path_buf()],
        recursive: true,
    };

    let result = cli.validate(&config).expect("validate should succeed");
    assert_eq!(result.total_files, 2);
    assert_eq!(result.files_with_cards, 1);
}

#[test]
fn test_validate_all_files_have_cards() {
    let dir = setup_temp_dir_with_files(&[("a.md", "- Q1 -> A1\n"), ("b.md", "- Q2 -> A2\n")]);

    let cli = AnkiCli::new();
    let config = acli::ValidationConfig {
        source_dirs: vec![dir.path().to_path_buf()],
        recursive: true,
    };

    let result = cli.validate(&config).expect("validate should succeed");
    assert_eq!(result.total_files, 2);
    assert_eq!(result.files_with_cards, 2);
}

#[test]
fn test_validate_only_files_without_cards() {
    let dir = setup_temp_dir_with_files(&[
        ("readme.md", "# Just notes\n"),
        ("notes.md", "Some plain text\n"),
    ]);

    let cli = AnkiCli::new();
    let config = acli::ValidationConfig {
        source_dirs: vec![dir.path().to_path_buf()],
        recursive: true,
    };

    let result = cli
        .validate(&config)
        .expect("validate should succeed even with no cards");
    assert_eq!(result.total_files, 2);
    assert_eq!(result.files_with_cards, 0);
}

#[test]
fn test_validate_fails_on_real_parse_errors() {
    // A file with invalid media filename should still cause a validation error
    let dir =
        setup_temp_dir_with_files(&[("bad.md", "- Question -> Answer ![alt](\"badfile\".png)\n")]);

    let cli = AnkiCli::new();
    let config = acli::ValidationConfig {
        source_dirs: vec![dir.path().to_path_buf()],
        recursive: true,
    };

    let result = cli.validate(&config);
    assert!(
        result.is_err(),
        "validate should fail on invalid media filenames"
    );
}

#[test]
fn test_sync_skips_files_without_cards() {
    let dir = setup_temp_dir_with_files(&[
        ("cards.md", "- Question -> Answer\n"),
        ("readme.md", "# No cards here\n"),
    ]);

    let cli = AnkiCli::new();
    let config = SyncConfig {
        source_dirs: vec![dir.path().to_path_buf()],
        deck_name: "Test".to_string(),
        anki_collection_path: None,
        anki_media_dir: None,
        recursive: true,
        dry_run: true,
    };

    let result = cli
        .sync(&config)
        .expect("sync should succeed with mixed files");
    assert_eq!(result.files_processed, 2);
    assert_eq!(result.cards_synced, 1);
}

// =============================================================================
// Test: Block cards
// =============================================================================

#[test]
fn test_sync_block_cards() {
    let content = r#"
- 30 ml Cognac
- 30 ml Crème de Cacao
- 30 ml Fresh Cream
->
Alexander

- 30 ml Campari
- 30 ml Sweet Vermouth
- splash Soda Water
->
Americano
"#;

    let temp_dir = setup_temp_dir_with_files(&[("cocktails.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Cocktails");

    let deck = adapter
        .inner()
        .decks
        .get("Cocktails")
        .expect("deck not found");
    assert_eq!(deck.len(), 2, "expected 2 block cards");

    // Verify both cocktail cards exist with correct answers
    let alexander = deck.iter().find(|c| c.fields[1].contains("Alexander"));
    assert!(alexander.is_some(), "Alexander card not found");

    let americano = deck.iter().find(|c| c.fields[1].contains("Americano"));
    assert!(americano.is_some(), "Americano card not found");

    // Verify question contains ingredients as a list
    let q = &alexander.unwrap().fields[0];
    assert!(q.contains("Cognac"), "question should contain Cognac: {q}");
    assert!(
        q.contains("<li>"),
        "question should be rendered as list: {q}"
    );
}

// =============================================================================
// Test: Templates with block cards
// =============================================================================

#[test]
fn test_sync_template_block_cards() {
    let content = "\
```template\n\
{{ ingredients }}\n\
->\n\
{{ name }}\n\
```\n\
\n\
name | ingredients\n\
Alexander | 30 ml Cognac, 30 ml Crème de Cacao, 30 ml Fresh Cream\n\
Americano | 30 ml Campari, 30 ml Sweet Vermouth, splash Soda Water\n\
Angel Face | 30 ml Gin, 30 ml Apricot Brandy, 30 ml Calvados";

    let temp_dir = setup_temp_dir_with_files(&[("cocktails.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Cocktails");

    let deck = adapter
        .inner()
        .decks
        .get("Cocktails")
        .expect("deck not found");
    assert_eq!(deck.len(), 3, "expected 3 cards from template");

    // Each card's answer should be the cocktail name
    let names: Vec<&str> = vec!["Alexander", "Americano", "Angel Face"];
    for name in &names {
        let card = deck.iter().find(|c| c.fields[1].contains(name));
        assert!(card.is_some(), "card for '{}' not found", name);
    }

    // Each card's question should contain the ingredients
    let alexander = deck
        .iter()
        .find(|c| c.fields[1].contains("Alexander"))
        .unwrap();
    assert!(
        alexander.fields[0].contains("Cognac"),
        "question should contain ingredients"
    );
}

#[test]
fn test_sync_template_inline_cards() {
    let content = "\
```template\n\
- {{ english }} <-> {{ spanish }}\n\
```\n\
\n\
english | spanish\n\
hello | hola\n\
goodbye | adiós\n\
good morning | buenos días";

    let temp_dir = setup_temp_dir_with_files(&[("vocab.md", content)]);
    let adapter = run_sync_pipeline(temp_dir.path(), "Vocab");

    let deck = adapter.inner().decks.get("Vocab").expect("deck not found");
    assert_eq!(
        deck.len(),
        3,
        "expected 3 bidirectional cards from template"
    );
}
