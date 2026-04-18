use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub const ENV_PANE_ID: &str = "XSHELL_PANE_ID";
pub const ENV_SESSION_ID: &str = "XSHELL_SESSION_ID";

/// Lit `stdin` (supposé JSON Claude Code hook), enrichit avec pane_id + session_id + kind,
/// puis publie une ligne JSON sur le socket. Blocking, synchrone.
pub fn forward(kind: &str, socket: &Path) -> std::io::Result<()> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw)?;

    let mut value: serde_json::Value =
        serde_json::from_str(raw.trim()).unwrap_or_else(|_| serde_json::json!({}));

    if !value.is_object() {
        value = serde_json::json!({});
    }

    let obj = value.as_object_mut().expect("object");
    obj.insert("kind".into(), serde_json::Value::String(kind.into()));

    if let Ok(pane_id) = std::env::var(ENV_PANE_ID) {
        if let Ok(n) = pane_id.parse::<u64>() {
            obj.insert("pane_id".into(), serde_json::Value::Number(n.into()));
        }
    }
    if let Ok(sid) = std::env::var(ENV_SESSION_ID) {
        obj.insert("session_id".into(), serde_json::Value::String(sid));
    }

    let mut line = serde_json::to_string(&value)?;
    line.push('\n');

    match UnixStream::connect(socket) {
        Ok(mut stream) => stream.write_all(line.as_bytes()),
        // Si l'app Xshell-AI n'est pas lancée, on ne veut pas faire échouer Claude Code.
        Err(_) => Ok(()),
    }
}

/// Map CLI arg → kind dans la convention du crate agent-manager (`kebab-case`).
pub fn kind_from_arg(arg: &str) -> Option<&'static str> {
    match arg {
        "session-start" => Some("session-start"),
        "prompt-submit" => Some("prompt-submit"),
        "pre-tool" => Some("pre-tool"),
        "post-tool" => Some("post-tool"),
        "stop" => Some("stop"),
        "session-end" => Some("session-end"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_from_arg_maps_known_kinds() {
        assert_eq!(kind_from_arg("session-start"), Some("session-start"));
        assert_eq!(kind_from_arg("post-tool"), Some("post-tool"));
        assert_eq!(kind_from_arg("bogus"), None);
    }
}
