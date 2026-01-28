use update_planner_parser::types::{Card, CardType};
use update_planner_parser::{
    DefaultExecutor, DocumentSet, PlanExecutor, SimplePlanner, UpdatePlanner,
};

struct MockCollection;

impl update_planner_parser::executor::AnkiCollection for MockCollection {
    fn ensure_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("ensure_deck: {}", deck_name);
        Ok(())
    }
    fn clear_deck(&mut self, deck_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("clear_deck: {}", deck_name);
        Ok(())
    }
    fn add_card(
        &mut self,
        deck_name: &str,
        card: &update_planner_parser::types::Card,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("add_card to {}: {:?}", deck_name, card);
        Ok(())
    }
    fn delete_card(
        &mut self,
        deck_name: &str,
        card_id: update_planner_parser::types::CardId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("delete_card from {}: {}", deck_name, card_id);
        Ok(())
    }
    fn update_card(
        &mut self,
        deck_name: &str,
        card_id: update_planner_parser::types::CardId,
        card: &update_planner_parser::types::Card,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("update_card in {}: {} -> {:?}", deck_name, card_id, card);
        Ok(())
    }
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("save");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docs = DocumentSet {
        cards: vec![
            Card {
                card_type: CardType::Basic,
                fields: vec!["Q".into(), "A".into()],
            },
            Card {
                card_type: CardType::Bidirectional,
                fields: vec!["Front".into(), "Back".into()],
            },
        ],
        source_files: vec!["example.md".into()],
    };

    let planner = SimplePlanner;
    let plan = planner.plan_fresh_sync(&docs, "DemoDeck")?;

    let exec = DefaultExecutor;
    let mut coll = MockCollection;
    exec.execute_plan(&plan, &mut coll)?;
    Ok(())
}
