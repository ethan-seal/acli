pub mod adapter;
pub mod cli;
pub mod discovery;
pub mod error;
pub mod media;
pub mod output;
pub mod sync;

pub use crate::adapter::AnkiCollectionAdapter;
pub use crate::cli::run;
pub use crate::output::{CliOutput, SyncResult};
pub use crate::sync::{sync_incremental, AnkiCli, SyncConfig, SyncCounts, ValidationConfig};
