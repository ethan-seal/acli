pub mod cli;
pub mod discovery;
pub mod doc_parser;
pub mod error;
pub mod output;
pub mod sync;

pub use crate::cli::run;
pub use crate::output::{CliOutput, SyncResult};
pub use crate::sync::{AnkiCli, SyncConfig, ValidationConfig};
