use crate::error::{Result, ThemeError};
use crate::parser::parse_file;
use crate::ColorScheme;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Racine des thèmes. Respecte `$XDG_CONFIG_HOME`.
pub fn themes_dir() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Ok(PathBuf::from(xdg).join("xshell-ai").join("themes"));
        }
    }
    let home = dirs::home_dir().ok_or(ThemeError::NoHome)?;
    Ok(home.join(".config").join("xshell-ai").join("themes"))
}

fn theme_path(name: &str) -> Result<PathBuf> {
    let sanitized: String = name
        .chars()
        .map(|c| if c == '/' || c == '\\' || c == '\0' { '_' } else { c })
        .collect();
    Ok(themes_dir()?.join(format!("{sanitized}.itermcolors")))
}

/// Enregistre le contenu brut d'un `.itermcolors` sous un nom donné.
pub fn save_raw(name: &str, bytes: &[u8]) -> Result<PathBuf> {
    let dir = themes_dir()?;
    fs::create_dir_all(&dir).map_err(|source| ThemeError::Io {
        path: dir.clone(),
        source,
    })?;
    let path = theme_path(name)?;
    let tmp = {
        let mut p = path.as_os_str().to_os_string();
        p.push(".tmp");
        PathBuf::from(p)
    };
    {
        let mut file = fs::File::create(&tmp).map_err(|source| ThemeError::Io {
            path: tmp.clone(),
            source,
        })?;
        file.write_all(bytes).map_err(|source| ThemeError::Io {
            path: tmp.clone(),
            source,
        })?;
        file.sync_all().map_err(|source| ThemeError::Io {
            path: tmp.clone(),
            source,
        })?;
    }
    fs::rename(&tmp, &path).map_err(|source| ThemeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

/// Charge et parse un thème par son nom.
pub fn load(name: &str) -> Result<ColorScheme> {
    let path = theme_path(name)?;
    if !path.exists() {
        return Err(ThemeError::NotFound(name.to_string()));
    }
    parse_file(&path, name)
}

/// Liste les thèmes stockés, triés.
pub fn list() -> Result<Vec<String>> {
    let dir = themes_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| ThemeError::Io {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ThemeError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("itermcolors") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            names.push(stem.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// Supprime un thème.
pub fn delete(name: &str) -> Result<()> {
    let path = theme_path(name)?;
    if !path.exists() {
        return Err(ThemeError::NotFound(name.to_string()));
    }
    fs::remove_file(&path).map_err(|source| ThemeError::Io { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use tempfile::tempdir;

    fn fixture_bytes() -> Vec<u8> {
        fs::read("tests/fixtures/dracula.itermcolors").expect("fixture")
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        save_raw("dracula", &fixture_bytes()).expect("save");
        let scheme = load("dracula").expect("load");
        assert_eq!(scheme.name, "dracula");
        assert_eq!(scheme.background, Color::new(40, 42, 54));
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn list_is_sorted() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        for n in ["zeta", "alpha", "mu"] {
            save_raw(n, &fixture_bytes()).expect("save");
        }
        let names = list().expect("list");
        assert_eq!(names, vec!["alpha", "mu", "zeta"]);
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn delete_removes_and_missing_errors() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        save_raw("scratch", &fixture_bytes()).expect("save");
        delete("scratch").expect("delete");
        assert!(matches!(
            delete("scratch").unwrap_err(),
            ThemeError::NotFound(_)
        ));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
