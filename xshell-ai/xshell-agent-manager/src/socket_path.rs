use std::path::PathBuf;

/// Chemin par défaut du socket UNIX : `/tmp/xshell-ai-{uid}.sock`.
/// Respecte `$XSHELL_SOCK` si défini (pour tests).
pub fn default_socket_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("XSHELL_SOCK") {
        if !override_path.is_empty() {
            return PathBuf::from(override_path);
        }
    }
    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/tmp/xshell-ai-{uid}.sock"))
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
    fn override_via_env_is_respected() {
        let _g = EnvGuard::set("XSHELL_SOCK", "/tmp/test.sock");
        assert_eq!(default_socket_path(), PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn default_is_uid_scoped() {
        let _g = EnvGuard::set("XSHELL_SOCK", "");
        let p = default_socket_path();
        let s = p.to_string_lossy();
        assert!(s.starts_with("/tmp/xshell-ai-"));
        assert!(s.ends_with(".sock"));
    }
}
