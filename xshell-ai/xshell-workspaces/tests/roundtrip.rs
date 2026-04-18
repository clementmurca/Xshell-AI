use chrono::{TimeZone, Utc};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tempfile::tempdir;
use xshell_workspaces::{
    delete, list, load_named, save_named, PaneConfig, TabConfig, UiConfig, WindowConfig,
    WorkspaceConfig,
};

fn full_workspace(name: &str, cwd: PathBuf) -> WorkspaceConfig {
    let mut env = BTreeMap::new();
    env.insert("KEY".to_string(), "value".to_string());

    WorkspaceConfig {
        name: name.to_string(),
        created_at: Utc.with_ymd_and_hms(2026, 4, 18, 10, 0, 0).unwrap(),
        last_opened_at: Utc.with_ymd_and_hms(2026, 4, 18, 12, 0, 0).unwrap(),
        ui: UiConfig {
            left_sidebar_open: true,
            right_sidebar_open: true,
            theme: Some("gruvbox".into()),
        },
        windows: vec![WindowConfig {
            id: 0,
            active_tab: 0,
            tabs: vec![TabConfig {
                id: 0,
                title: "main".into(),
                layout: "single".into(),
                panes: vec![PaneConfig {
                    cwd,
                    cmd: Some("claude".into()),
                    env,
                }],
            }],
        }],
    }
}

#[test]
fn create_save_list_load_delete_cycle() {
    let tmp = tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", tmp.path());

    let ws1 = full_workspace("alpha", tmp.path().to_path_buf());
    let ws2 = full_workspace("beta", tmp.path().to_path_buf());

    save_named(&ws1).unwrap();
    save_named(&ws2).unwrap();

    let names = list().unwrap();
    assert_eq!(names, vec!["alpha", "beta"]);

    let (loaded, warnings) = load_named("alpha").unwrap();
    assert_eq!(loaded, ws1);
    assert!(warnings.is_empty());

    delete("alpha").unwrap();
    let names_after = list().unwrap();
    assert_eq!(names_after, vec!["beta"]);

    std::env::remove_var("XDG_CONFIG_HOME");
}
