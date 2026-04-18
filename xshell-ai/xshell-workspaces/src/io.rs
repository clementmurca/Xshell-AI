use crate::config::WorkspaceConfig;
use crate::error::{Result, WorkspaceError};
use crate::paths;
use crate::paths::{last_session_path, workspace_path, workspaces_dir, LAST_SESSION_STEM};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sauvegarde un workspace sous son nom dans `workspaces_dir()`.
pub fn save_named(config: &WorkspaceConfig) -> Result<()> {
    let path = workspace_path(&config.name)?;
    save_to_path(config, &path)
}

/// Charge un workspace par son nom.
pub fn load_named(name: &str) -> Result<(WorkspaceConfig, Vec<String>)> {
    let path = workspace_path(name)?;
    if !path.exists() {
        return Err(WorkspaceError::NotFound(name.to_string()));
    }
    load_from_path(&path)
}

/// Liste les workspaces nommés, triés, en excluant `last-session`.
pub fn list() -> Result<Vec<String>> {
    let dir = workspaces_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| WorkspaceError::Io {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| WorkspaceError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if stem == LAST_SESSION_STEM {
            continue;
        }
        names.push(stem);
    }
    names.sort();
    Ok(names)
}

/// Supprime un workspace nommé. Erreur `NotFound` si absent.
pub fn delete(name: &str) -> Result<()> {
    let path = workspace_path(name)?;
    if !path.exists() {
        return Err(WorkspaceError::NotFound(name.to_string()));
    }
    fs::remove_file(&path).map_err(|source| WorkspaceError::Io { path, source })
}

/// Sauvegarde dans `last-session.toml` (helper pour l'auto-save).
pub fn save_last_session(config: &WorkspaceConfig) -> Result<()> {
    let path = last_session_path()?;
    save_to_path(config, &path)
}

/// Charge `last-session.toml` si présent, sinon `None`.
pub fn load_last_session() -> Result<Option<(WorkspaceConfig, Vec<String>)>> {
    let path = last_session_path()?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(load_from_path(&path)?))
}

/// Sauvegarde atomique : écrit un fichier temporaire voisin puis `rename`.
pub fn save_to_path(config: &WorkspaceConfig, path: &Path) -> Result<()> {
    paths::ensure_parent(path)?;

    let serialized = toml::to_string_pretty(config)?;

    let tmp = tmp_path_for(path);
    {
        let mut file = fs::File::create(&tmp).map_err(|source| WorkspaceError::Io {
            path: tmp.clone(),
            source,
        })?;
        file.write_all(serialized.as_bytes())
            .map_err(|source| WorkspaceError::Io {
                path: tmp.clone(),
                source,
            })?;
        file.sync_all().map_err(|source| WorkspaceError::Io {
            path: tmp.clone(),
            source,
        })?;
    }

    fs::rename(&tmp, path).map_err(|source| WorkspaceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Charge un workspace depuis un chemin. Les `cwd` qui n'existent plus sont
/// remplacés par `$HOME` et signalés dans le `Vec<String>` retourné.
pub fn load_from_path(path: &Path) -> Result<(WorkspaceConfig, Vec<String>)> {
    let raw = fs::read_to_string(path).map_err(|source| WorkspaceError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut config: WorkspaceConfig =
        toml::from_str(&raw).map_err(|source| WorkspaceError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

    let home = dirs::home_dir().ok_or(WorkspaceError::NoHome)?;
    let mut warnings = Vec::new();

    for window in &mut config.windows {
        for tab in &mut window.tabs {
            for pane in &mut tab.panes {
                if !pane.cwd.exists() {
                    warnings.push(format!(
                        "cwd '{}' missing, fallback to HOME",
                        pane.cwd.display()
                    ));
                    pane.cwd = home.clone();
                }
            }
        }
    }

    Ok((config, warnings))
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{UiConfig, WorkspaceConfig};
    use chrono::Utc;
    use tempfile::tempdir;

    fn minimal() -> WorkspaceConfig {
        let now = Utc::now();
        WorkspaceConfig {
            name: "test".into(),
            created_at: now,
            last_opened_at: now,
            ui: UiConfig {
                left_sidebar_open: false,
                right_sidebar_open: false,
                theme: None,
            },
            windows: vec![],
        }
    }

    #[test]
    fn save_writes_file_to_path() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.toml");
        save_to_path(&minimal(), &path).expect("save");
        assert!(path.exists());
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("name = \"test\""));
    }

    #[test]
    fn save_cleans_up_tmp_after_rename() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.toml");
        save_to_path(&minimal(), &path).expect("save");
        assert!(!tmp_path_for(&path).exists());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/deep/ws.toml");
        save_to_path(&minimal(), &path).expect("save");
        assert!(path.exists());
    }

    use crate::config::{PaneConfig, TabConfig, WindowConfig};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn one_pane_missing_cwd() -> WorkspaceConfig {
        let now = Utc::now();
        WorkspaceConfig {
            name: "test".into(),
            created_at: now,
            last_opened_at: now,
            ui: UiConfig {
                left_sidebar_open: false,
                right_sidebar_open: false,
                theme: None,
            },
            windows: vec![WindowConfig {
                id: 0,
                active_tab: 0,
                tabs: vec![TabConfig {
                    id: 0,
                    title: "t".into(),
                    layout: "single".into(),
                    panes: vec![PaneConfig {
                        cwd: PathBuf::from("/nonexistent/path/should/not/exist"),
                        cmd: None,
                        env: BTreeMap::new(),
                    }],
                }],
            }],
        }
    }

    #[test]
    fn load_replaces_missing_cwd_with_home() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.toml");
        save_to_path(&one_pane_missing_cwd(), &path).expect("save");

        let (loaded, warnings) = load_from_path(&path).expect("load");
        let home = dirs::home_dir().expect("home");

        assert_eq!(loaded.windows[0].tabs[0].panes[0].cwd, home);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("/nonexistent/path/should/not/exist"));
    }

    #[test]
    fn load_keeps_existing_cwd_unchanged() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ws.toml");
        let mut config = one_pane_missing_cwd();
        config.windows[0].tabs[0].panes[0].cwd = dir.path().to_path_buf();
        save_to_path(&config, &path).expect("save");

        let (loaded, warnings) = load_from_path(&path).expect("load");
        assert_eq!(loaded.windows[0].tabs[0].panes[0].cwd, dir.path());
        assert!(warnings.is_empty());
    }

    #[test]
    fn load_errors_on_malformed_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "this is = not valid = toml = at all").unwrap();
        let err = load_from_path(&path).unwrap_err();
        assert!(matches!(err, WorkspaceError::Parse { .. }));
    }

    use super::{delete, list, load_named, save_named};

    #[test]
    fn save_named_and_load_named_roundtrip() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let mut config = minimal();
        config.name = "backend".into();
        save_named(&config).expect("save_named");

        let (loaded, warnings) = load_named("backend").expect("load_named");
        assert_eq!(loaded.name, "backend");
        assert!(warnings.is_empty());

        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn list_returns_sorted_stems_without_last_session() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        for name in ["zeta", "alpha", "mu"] {
            let mut c = minimal();
            c.name = name.into();
            save_named(&c).expect("save_named");
        }
        let mut c = minimal();
        c.name = "last-session".into();
        save_named(&c).expect("save_named");

        let names = list().expect("list");
        assert_eq!(names, vec!["alpha", "mu", "zeta"]);

        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn delete_removes_file_and_missing_errors() {
        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let mut c = minimal();
        c.name = "scratch".into();
        save_named(&c).expect("save_named");
        delete("scratch").expect("delete");

        let err = delete("scratch").unwrap_err();
        assert!(matches!(err, WorkspaceError::NotFound(_)));

        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
