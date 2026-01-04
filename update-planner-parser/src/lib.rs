#![deny(missing_docs)]
//! Update planner library: creates sync plans from parsed documents.

pub mod error;
pub mod types;
pub mod planner;
pub mod executor;

pub use crate::error::PlanError;
pub use crate::types::{CardId, Operation, SyncPlan, DocumentSet};
pub use crate::planner::{UpdatePlanner, SimplePlanner};
pub use crate::executor::{PlanExecutor, DefaultExecutor};
