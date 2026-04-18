use crate::error::{Result, WorkspaceError};
use std::path::{Path, PathBuf};

pub const LAST_SESSION_STEM: &str = "last-session";

/// Racine de config Xshell-AI. Respecte `$XDG_CONFIG_HOME`, sinon `~/.config/xshell-ai`.
pub fn config_root() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Ok(PathBuf::from(xdg).join("xshell-ai"));
        }
    }
    let home = dirs::home_dir().ok_or(WorkspaceError::NoHome)?;
    Ok(home.join(".config").join("xshell-ai"))
}

/// Répertoire où vivent les TOML nommés.
pub fn workspaces_dir() -> Result<PathBuf> {
    Ok(config_root()?.join("workspaces"))
}

/// Chemin d'un workspace nommé.
pub fn workspace_path(name: &str) -> Result<PathBuf> {
    Ok(workspaces_dir()?.join(format!("{}.toml", sanitize(name))))
}

/// Chemin du `last-session.toml`.
pub fn last_session_path() -> Result<PathBuf> {
    Ok(workspaces_dir()?.join(format!("{LAST_SESSION_STEM}.toml")))
}

/// Crée les répertoires parents d'un fichier workspace si absents.
pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| WorkspaceError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' || c == '\\' || c == '\0' { '_' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn config_root_uses_xdg_env_when_set() {
        let _guard = EnvGuard::set("XDG_CONFIG_HOME", "/tmp/fake-xdg");
        let root = config_root().expect("root");
        assert_eq!(root, PathBuf::from("/tmp/fake-xdg/xshell-ai"));
    }

    #[test]
    fn workspace_path_sanitizes_slashes() {
        let _guard = EnvGuard::set("XDG_CONFIG_HOME", "/tmp/fake-xdg");
        let path = workspace_path("evil/../name").expect("path");
        assert_eq!(
            path,
            PathBuf::from("/tmp/fake-xdg/xshell-ai/workspaces/evil_.._name.toml")
        );
    }

    #[test]
    fn last_session_path_ends_with_last_session_toml() {
        let _guard = EnvGuard::set("XDG_CONFIG_HOME", "/tmp/fake-xdg");
        let path = last_session_path().expect("path");
        assert!(path.ends_with("workspaces/last-session.toml"));
    }
}
