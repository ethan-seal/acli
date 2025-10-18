use crate::collection::{AnkiCollection, FakeAnkiCollection};
use crate::types::{Card, CardType, DeckConfig};

#[test]
fn fake_collection_adds_and_clears_cards() {
    let mut col = FakeAnkiCollection::new();
    let deck = DeckConfig {
        name: "Test".to_string(),
    };

    col.ensure_deck(&deck).unwrap();

    let c1 = Card {
        card_type: CardType::Basic,
        fields: vec!["Front".into(), "Back".into()],
    };
    col.add_card(&deck, &c1).unwrap();

    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 1);

    col.clear_deck(&deck).unwrap();
    assert_eq!(col.decks.get(&deck.name).unwrap().len(), 0);

    col.save().unwrap();
    assert!(col.save_called);
}
