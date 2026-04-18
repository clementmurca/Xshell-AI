pub mod config;
pub mod error;
pub mod io;
pub mod paths;

pub use config::{PaneConfig, TabConfig, UiConfig, WindowConfig, WorkspaceConfig};
pub use error::{Result, WorkspaceError};
pub use io::{
    delete, list, load_from_path, load_last_session, load_named, save_last_session, save_named,
    save_to_path,
};
