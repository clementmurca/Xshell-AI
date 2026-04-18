use crate::events::{HookEvent, PaneId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Statut dérivé d'un agent, consommé par la sidebar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AgentStatus {
    Idle,
    Thinking,
    WaitingInput,
    ExecutingTool { tool: String },
    Stopped,
}

/// État d'un agent Claude dans un pane donné.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub pane_id: PaneId,
    pub session_id: String,
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub status: AgentStatus,
    pub tokens_used: Option<usize>,
    pub last_output: Option<String>,
    pub files_modified: HashSet<PathBuf>,
    pub tool_call_count: usize,
    pub started_at: DateTime<Utc>,
}

impl AgentState {
    pub fn new(
        pane_id: PaneId,
        session_id: String,
        cwd: PathBuf,
        model: Option<String>,
        started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            pane_id,
            session_id,
            cwd,
            model,
            status: AgentStatus::Idle,
            tokens_used: None,
            last_output: None,
            files_modified: HashSet::new(),
            tool_call_count: 0,
            started_at,
        }
    }
}

/// Store en mémoire — à wrap dans un `Arc<Mutex<_>>` ou un acteur tokio côté serveur.
#[derive(Debug, Default, Clone)]
pub struct AgentStore {
    pub agents: HashMap<PaneId, AgentState>,
}

impl AgentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, pane_id: PaneId) -> Option<&AgentState> {
        self.agents.get(&pane_id)
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Applique un event au store. Retourne `true` si l'état de l'agent concerné a changé.
    pub fn apply(&mut self, event: HookEvent) -> bool {
        apply_event(self, event, Utc::now())
    }
}

/// Transition explicite : `event → mutation du store`. `now` injecté pour tests déterministes.
pub fn apply_event(store: &mut AgentStore, event: HookEvent, now: DateTime<Utc>) -> bool {
    match event {
        HookEvent::SessionStart {
            pane_id,
            session_id,
            cwd,
            model,
        } => {
            let state = AgentState::new(pane_id, session_id, cwd, model, now);
            store.agents.insert(pane_id, state);
            true
        }
        HookEvent::PromptSubmit { pane_id, .. } => {
            if let Some(s) = store.agents.get_mut(&pane_id) {
                s.status = AgentStatus::Thinking;
                true
            } else {
                false
            }
        }
        HookEvent::PreTool {
            pane_id, tool, ..
        } => {
            if let Some(s) = store.agents.get_mut(&pane_id) {
                s.status = AgentStatus::ExecutingTool { tool };
                s.tool_call_count += 1;
                true
            } else {
                false
            }
        }
        HookEvent::PostTool {
            pane_id,
            modified_files,
            ..
        } => {
            if let Some(s) = store.agents.get_mut(&pane_id) {
                s.status = AgentStatus::Thinking;
                for f in modified_files {
                    s.files_modified.insert(f);
                }
                true
            } else {
                false
            }
        }
        HookEvent::Stop {
            pane_id,
            last_output,
            tokens_used,
            ..
        } => {
            if let Some(s) = store.agents.get_mut(&pane_id) {
                s.status = AgentStatus::WaitingInput;
                if let Some(out) = last_output {
                    s.last_output = Some(truncate(&out, 120));
                }
                if let Some(t) = tokens_used {
                    s.tokens_used = Some(t);
                }
                true
            } else {
                false
            }
        }
        HookEvent::SessionEnd { pane_id, .. } => store.agents.remove(&pane_id).is_some(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{truncated}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 18, 10, 0, 0).unwrap()
    }

    fn start(pane: PaneId) -> HookEvent {
        HookEvent::SessionStart {
            pane_id: pane,
            session_id: "s".into(),
            cwd: PathBuf::from("/tmp"),
            model: Some("claude-opus-4-7".into()),
        }
    }

    #[test]
    fn session_start_creates_agent_with_idle_status() {
        let mut store = AgentStore::new();
        let changed = apply_event(&mut store, start(1), t0());
        assert!(changed);
        assert_eq!(store.len(), 1);
        let state = store.get(1).unwrap();
        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.started_at, t0());
    }

    #[test]
    fn full_lifecycle_idle_to_tool_to_waiting() {
        let mut store = AgentStore::new();
        apply_event(&mut store, start(1), t0());
        apply_event(
            &mut store,
            HookEvent::PromptSubmit {
                pane_id: 1,
                session_id: "s".into(),
            },
            t0(),
        );
        assert_eq!(store.get(1).unwrap().status, AgentStatus::Thinking);

        apply_event(
            &mut store,
            HookEvent::PreTool {
                pane_id: 1,
                session_id: "s".into(),
                tool: "Edit".into(),
            },
            t0(),
        );
        assert_eq!(
            store.get(1).unwrap().status,
            AgentStatus::ExecutingTool { tool: "Edit".into() }
        );
        assert_eq!(store.get(1).unwrap().tool_call_count, 1);

        apply_event(
            &mut store,
            HookEvent::PostTool {
                pane_id: 1,
                session_id: "s".into(),
                tool: "Edit".into(),
                modified_files: vec![PathBuf::from("/a.rs"), PathBuf::from("/b.rs")],
            },
            t0(),
        );
        assert_eq!(store.get(1).unwrap().status, AgentStatus::Thinking);
        assert_eq!(store.get(1).unwrap().files_modified.len(), 2);

        apply_event(
            &mut store,
            HookEvent::Stop {
                pane_id: 1,
                session_id: "s".into(),
                last_output: Some("done".into()),
                tokens_used: Some(1234),
            },
            t0(),
        );
        let s = store.get(1).unwrap();
        assert_eq!(s.status, AgentStatus::WaitingInput);
        assert_eq!(s.last_output.as_deref(), Some("done"));
        assert_eq!(s.tokens_used, Some(1234));
    }

    #[test]
    fn session_end_removes_agent() {
        let mut store = AgentStore::new();
        apply_event(&mut store, start(1), t0());
        assert_eq!(store.len(), 1);
        apply_event(
            &mut store,
            HookEvent::SessionEnd {
                pane_id: 1,
                session_id: "s".into(),
            },
            t0(),
        );
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn events_for_unknown_pane_are_ignored() {
        let mut store = AgentStore::new();
        let changed = apply_event(
            &mut store,
            HookEvent::PromptSubmit {
                pane_id: 99,
                session_id: "s".into(),
            },
            t0(),
        );
        assert!(!changed);
        assert!(store.is_empty());
    }

    #[test]
    fn last_output_truncated_to_120_chars() {
        let mut store = AgentStore::new();
        apply_event(&mut store, start(1), t0());
        let long = "x".repeat(200);
        apply_event(
            &mut store,
            HookEvent::Stop {
                pane_id: 1,
                session_id: "s".into(),
                last_output: Some(long),
                tokens_used: None,
            },
            t0(),
        );
        let out = store.get(1).unwrap().last_output.as_ref().unwrap();
        assert!(out.chars().count() <= 121);
        assert!(out.ends_with('…'));
    }
}
