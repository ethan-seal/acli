#![deny(missing_docs)]
//! Update planner library: creates sync plans from parsed documents.

pub mod error;
pub mod types;
pub mod planner;

pub use crate::error::PlanError;
pub use crate::types::{Operation, SyncPlan, DocumentSet};
pub use crate::planner::{UpdatePlanner, SimplePlanner};
