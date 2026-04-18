use crate::error::{AgentError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Identifiant d'un pane Wezterm (type opaque côté Agent Manager).
pub type PaneId = u64;

/// Event provenant d'un hook Claude Code, après parsing et validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum HookEvent {
    SessionStart {
        pane_id: PaneId,
        session_id: String,
        cwd: PathBuf,
        model: Option<String>,
    },
    PromptSubmit {
        pane_id: PaneId,
        session_id: String,
    },
    PreTool {
        pane_id: PaneId,
        session_id: String,
        tool: String,
    },
    PostTool {
        pane_id: PaneId,
        session_id: String,
        tool: String,
        modified_files: Vec<PathBuf>,
    },
    Stop {
        pane_id: PaneId,
        session_id: String,
        last_output: Option<String>,
        tokens_used: Option<usize>,
    },
    SessionEnd {
        pane_id: PaneId,
        session_id: String,
    },
}

impl HookEvent {
    pub fn pane_id(&self) -> PaneId {
        match self {
            HookEvent::SessionStart { pane_id, .. }
            | HookEvent::PromptSubmit { pane_id, .. }
            | HookEvent::PreTool { pane_id, .. }
            | HookEvent::PostTool { pane_id, .. }
            | HookEvent::Stop { pane_id, .. }
            | HookEvent::SessionEnd { pane_id, .. } => *pane_id,
        }
    }
}

/// Parse une ligne JSON reçue du hook en `HookEvent`.
pub fn parse_event(line: &str) -> Result<HookEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(AgentError::MissingField("kind"));
    }
    let event: HookEvent = serde_json::from_str(trimmed)?;
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_start() {
        let json = r#"{
            "kind": "session-start",
            "pane_id": 42,
            "session_id": "abc",
            "cwd": "/tmp",
            "model": "claude-opus-4-7"
        }"#;
        let ev = parse_event(json).expect("parse");
        assert_eq!(
            ev,
            HookEvent::SessionStart {
                pane_id: 42,
                session_id: "abc".into(),
                cwd: PathBuf::from("/tmp"),
                model: Some("claude-opus-4-7".into()),
            }
        );
    }

    #[test]
    fn parse_post_tool_with_files() {
        let json = r#"{
            "kind": "post-tool",
            "pane_id": 1,
            "session_id": "s",
            "tool": "Edit",
            "modified_files": ["/a.rs", "/b.rs"]
        }"#;
        let ev = parse_event(json).expect("parse");
        match ev {
            HookEvent::PostTool { modified_files, tool, .. } => {
                assert_eq!(tool, "Edit");
                assert_eq!(modified_files.len(), 2);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn parse_rejects_empty_line() {
        let err = parse_event("").unwrap_err();
        assert!(matches!(err, AgentError::MissingField(_)));
    }

    #[test]
    fn parse_rejects_unknown_kind() {
        let err = parse_event(r#"{"kind": "bogus", "pane_id": 1, "session_id": "s"}"#)
            .unwrap_err();
        assert!(matches!(err, AgentError::Parse(_)));
    }

    #[test]
    fn pane_id_returns_value_from_any_variant() {
        let ev = HookEvent::Stop {
            pane_id: 7,
            session_id: "s".into(),
            last_output: None,
            tokens_used: None,
        };
        assert_eq!(ev.pane_id(), 7);
    }
}
