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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckConfig {
    pub name: String,
}
