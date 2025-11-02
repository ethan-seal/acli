/// Opaque card identifier (wraps i64 from Anki)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CardId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    Basic,
    BasicReversed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub card_type: CardType,
    pub fields: Vec<String>,
}

/// Information about a card in the collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardInfo {
    pub id: CardId,
    pub card_type: CardType,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckConfig {
    pub name: String,
}
