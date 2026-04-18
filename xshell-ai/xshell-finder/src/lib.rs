pub mod error;
pub mod path_utils;

pub use error::{FinderError, Result};
pub use path_utils::{normalise_title, resolve_default_path};

use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
mod dialog;

/// Ouvre un dialogue natif de sélection de dossier.
///
/// Retourne `Ok(Some(path))` si l'utilisateur a validé, `Ok(None)` s'il a annulé.
/// `Err` seulement si la plateforme ne supporte pas le dialog ou si `default_path` est invalide.
pub fn open_folder_dialog(
    title: &str,
    default_path: Option<&Path>,
) -> Result<Option<PathBuf>> {
    let title = normalise_title(title);
    let default_path = resolve_default_path(default_path)?;

    #[cfg(target_os = "macos")]
    {
        dialog::run_ns_open_panel(&title, default_path.as_deref())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, default_path);
        Err(FinderError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_returns_unsupported() {
        let err = open_folder_dialog("pick", None).unwrap_err();
        assert!(matches!(err, FinderError::Unsupported));
    }

    #[test]
    fn invalid_default_path_is_rejected_before_dispatch() {
        let err = open_folder_dialog("pick", Some(Path::new("/nope/42"))).unwrap_err();
        assert!(matches!(err, FinderError::InvalidDefaultPath(_)));
    }
}
