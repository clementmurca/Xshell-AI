use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FinderError {
    #[error("default path '{0}' does not exist or is not a directory")]
    InvalidDefaultPath(PathBuf),

    #[error("NSOpenPanel returned a URL that is not a local file path")]
    NonFileUrl,

    #[error("finder dialog unavailable on this platform")]
    Unsupported,
}

pub type Result<T> = std::result::Result<T, FinderError>;
