//! Planner implementations for generating sync plans.
use crate::error::PlanError;
use crate::types::{DocumentSet, Operation, SyncPlan};

/// Planner that produces sync plans from a document set.
pub trait UpdatePlanner {
    /// Planner error type
    type Error: core::fmt::Debug + core::fmt::Display;

    /// Generate a fresh sync plan adding all cards to `deck_name`.
    fn plan_fresh_sync(&self, documents: &DocumentSet, deck_name: &str) -> Result<SyncPlan, Self::Error>;

    /// Placeholder for future incremental planning.
    fn plan_incremental_sync(
        &self,
        _documents: &DocumentSet,
        _deck_name: &str,
    ) -> Result<SyncPlan, Self::Error>;
}

/// Simple planner: fresh sync only, adds all cards.
pub struct SimplePlanner;

impl UpdatePlanner for SimplePlanner {
    type Error = PlanError;

    fn plan_fresh_sync(&self, documents: &DocumentSet, deck_name: &str) -> Result<SyncPlan, Self::Error> {
        if documents.cards.is_empty() {
            return Err(PlanError::EmptyDocumentSet);
        }
        if deck_name.trim().is_empty() {
            return Err(PlanError::InvalidDeckName(deck_name.to_string()));
        }

        let operations = documents
            .cards
            .iter()
            .cloned()
            .map(Operation::Add)
            .collect();

        Ok(SyncPlan { operations, deck_name: deck_name.to_string() })
    }

    fn plan_incremental_sync(
        &self,
        _documents: &DocumentSet,
        deck_name: &str,
    ) -> Result<SyncPlan, Self::Error> {
        // Not implemented yet; return invalid deck name if applicable for consistency
        if deck_name.trim().is_empty() {
            return Err(PlanError::InvalidDeckName(deck_name.to_string()));
        }
        Err(PlanError::DuplicateCards(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Card, CardType, DocumentSet};

    #[test]
    fn test_fresh_sync_plan() {
        let cards = vec![
            Card { card_type: CardType::Basic, fields: vec!["Q".into(), "A".into()] },
            Card { card_type: CardType::Bidirectional, fields: vec!["F".into(), "B".into()] },
        ];

        let doc_set = DocumentSet { cards, source_files: vec!["test.md".into()] };
        let planner = SimplePlanner;
        let plan = planner.plan_fresh_sync(&doc_set, "TestDeck").unwrap();

        assert_eq!(plan.operations.len(), 2);
        assert_eq!(plan.deck_name, "TestDeck");
        assert!(matches!(plan.operations[0], Operation::Add(_)));
        assert!(matches!(plan.operations[1], Operation::Add(_)));
    }

    #[test]
    fn test_empty_document_set() {
        let doc_set = DocumentSet { cards: vec![], source_files: vec![] };
        let planner = SimplePlanner;
        let result = planner.plan_fresh_sync(&doc_set, "TestDeck");
        assert!(matches!(result, Err(PlanError::EmptyDocumentSet)));
    }

    #[test]
    fn test_invalid_deck_name() {
        let doc_set = DocumentSet {
            cards: vec![Card { card_type: CardType::Basic, fields: vec!["Q".into(), "A".into()] }],
            source_files: vec![],
        };
        let planner = SimplePlanner;
        let result = planner.plan_fresh_sync(&doc_set, "   ");
        assert!(matches!(result, Err(PlanError::InvalidDeckName(_))));
    }
}
