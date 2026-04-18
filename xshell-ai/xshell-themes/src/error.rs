use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse plist from {path}: {source}")]
    Plist {
        path: PathBuf,
        #[source]
        source: plist::Error,
    },

    #[error("iTerm2 color scheme missing required key: {0}")]
    MissingKey(String),

    #[error("invalid iTerm2 color component for key '{key}': {reason}")]
    InvalidColor { key: String, reason: String },

    #[error("download failed: {0}")]
    Download(#[from] reqwest::Error),

    #[error("download exceeded size limit of {limit} bytes")]
    SizeLimitExceeded { limit: usize },

    #[error("unexpected content-type '{0}' (expected xml or octet-stream)")]
    UnexpectedContentType(String),

    #[error("HOME directory could not be resolved")]
    NoHome,

    #[error("theme '{0}' does not exist")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ThemeError>;
