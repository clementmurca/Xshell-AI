use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse hook event: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("unknown hook event kind: '{0}'")]
    UnknownEvent(String),

    #[error("hook event missing required field: {0}")]
    MissingField(&'static str),
}

pub type Result<T> = std::result::Result<T, AgentError>;
