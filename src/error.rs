use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("feature not yet implemented")]
    NotImplemented,
}
