#[derive(Debug, Clone, PartialEq)]
pub enum CardType {
    Basic,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub card_type: CardType,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedDocument {
    pub cards: Vec<Card>,
    pub source_path: Option<String>,
}
