use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: DateTime<Utc>,
    pub ui: UiConfig,
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    pub left_sidebar_open: bool,
    pub right_sidebar_open: bool,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    pub id: u32,
    pub active_tab: u32,
    #[serde(default)]
    pub tabs: Vec<TabConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabConfig {
    pub id: u32,
    pub title: String,
    pub layout: String,
    #[serde(default)]
    pub panes: Vec<PaneConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneConfig {
    pub cwd: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_config() -> WorkspaceConfig {
        let mut env = BTreeMap::new();
        env.insert("PROJECT".to_string(), "backend".to_string());

        WorkspaceConfig {
            name: "refacto-backend".to_string(),
            created_at: Utc.with_ymd_and_hms(2026, 4, 17, 19, 30, 0).unwrap(),
            last_opened_at: Utc.with_ymd_and_hms(2026, 4, 17, 20, 15, 0).unwrap(),
            ui: UiConfig {
                left_sidebar_open: true,
                right_sidebar_open: false,
                theme: Some("tokyonight-storm".to_string()),
            },
            windows: vec![WindowConfig {
                id: 0,
                active_tab: 1,
                tabs: vec![TabConfig {
                    id: 0,
                    title: "db-schema".to_string(),
                    layout: "horizontal-split".to_string(),
                    panes: vec![
                        PaneConfig {
                            cwd: PathBuf::from("/Users/clement/projects/backend"),
                            cmd: Some("claude".to_string()),
                            env,
                        },
                        PaneConfig {
                            cwd: PathBuf::from("/Users/clement/projects/backend/migrations"),
                            cmd: None,
                            env: BTreeMap::new(),
                        },
                    ],
                }],
            }],
        }
    }

    #[test]
    fn toml_roundtrip_preserves_full_config() {
        let original = sample_config();
        let serialized = toml::to_string(&original).expect("serialize");
        let parsed: WorkspaceConfig = toml::from_str(&serialized).expect("parse");
        assert_eq!(parsed, original);
    }

    #[test]
    fn empty_env_is_omitted_from_toml() {
        let mut config = sample_config();
        config.windows[0].tabs[0].panes[1].env.clear();
        let serialized = toml::to_string(&config).expect("serialize");
        assert!(!serialized.contains("[windows.tabs.panes.env]\n]\n"));
    }

    #[test]
    fn null_cmd_is_omitted_from_toml() {
        let config = sample_config();
        let serialized = toml::to_string(&config).expect("serialize");
        let pane2_fragment = serialized
            .split("[[windows.tabs.panes]]")
            .nth(2)
            .expect("two panes");
        assert!(!pane2_fragment.contains("cmd"));
    }
}
