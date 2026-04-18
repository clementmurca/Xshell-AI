use crate::error::{FinderError, Result};
use std::path::{Path, PathBuf};

/// Valide le chemin par défaut fourni à `open_folder_dialog`.
pub fn resolve_default_path(p: Option<&Path>) -> Result<Option<PathBuf>> {
    let Some(path) = p else {
        return Ok(None);
    };
    if !path.exists() || !path.is_dir() {
        return Err(FinderError::InvalidDefaultPath(path.to_path_buf()));
    }
    Ok(Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf())))
}

/// Nettoie un titre : jamais vide, tronqué à 120 chars.
pub fn normalise_title(title: &str) -> String {
    let trimmed = title.trim();
    let base = if trimmed.is_empty() {
        "Select folder"
    } else {
        trimmed
    };
    base.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_default_none_returns_none() {
        assert_eq!(resolve_default_path(None).unwrap(), None);
    }

    #[test]
    fn resolve_default_existing_dir_returns_path() {
        let dir = tempdir().unwrap();
        let out = resolve_default_path(Some(dir.path())).expect("ok");
        assert!(out.is_some());
    }

    #[test]
    fn resolve_default_missing_path_errors() {
        let err = resolve_default_path(Some(Path::new("/nope/nowhere/42"))).unwrap_err();
        assert!(matches!(err, FinderError::InvalidDefaultPath(_)));
    }

    #[test]
    fn resolve_default_file_path_errors() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("f.txt");
        std::fs::write(&file, b"x").unwrap();
        let err = resolve_default_path(Some(&file)).unwrap_err();
        assert!(matches!(err, FinderError::InvalidDefaultPath(_)));
    }

    #[test]
    fn normalise_title_trims_and_defaults() {
        assert_eq!(normalise_title("  "), "Select folder");
        assert_eq!(normalise_title("  hello  "), "hello");
    }

    #[test]
    fn normalise_title_truncates_to_120_chars() {
        let long = "a".repeat(500);
        assert_eq!(normalise_title(&long).chars().count(), 120);
    }
}
