#![deny(missing_docs)]
//! Update planner library: creates sync plans from parsed documents.

pub mod error;
pub mod executor;
pub mod planner;
pub mod types;

pub use crate::error::PlanError;
pub use crate::executor::{DefaultExecutor, PlanExecutor};
pub use crate::planner::{SimplePlanner, UpdatePlanner};
pub use crate::types::{Card, CardId, CardType, DocumentDiff, DocumentSet, Operation, SyncPlan};
