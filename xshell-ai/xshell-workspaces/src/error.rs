use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize workspace to TOML: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("failed to parse workspace TOML from {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("HOME directory could not be resolved")]
    NoHome,

    #[error("workspace '{0}' does not exist")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;
