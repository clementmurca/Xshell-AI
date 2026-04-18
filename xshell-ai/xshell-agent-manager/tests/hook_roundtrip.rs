use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use xshell_agent_manager::{start_server, HookEvent};

#[tokio::test]
async fn full_session_lifecycle_over_socket() {
    let dir = tempdir().unwrap();
    let sock = dir.path().join("xs.sock");
    let (mut handle, _jh) = start_server(&sock).await.expect("start");

    async fn send(sock: &std::path::Path, ev: &HookEvent) {
        let mut s = UnixStream::connect(sock).await.expect("connect");
        let mut line = serde_json::to_string(ev).expect("serialize");
        line.push('\n');
        s.write_all(line.as_bytes()).await.expect("write");
        s.shutdown().await.ok();
    }

    send(
        &sock,
        &HookEvent::SessionStart {
            pane_id: 42,
            session_id: "abc".into(),
            cwd: PathBuf::from("/tmp"),
            model: Some("claude-opus-4-7".into()),
        },
    )
    .await;
    timeout(Duration::from_secs(2), handle.changed())
        .await
        .expect("not timed out")
        .expect("ok");

    send(
        &sock,
        &HookEvent::PreTool {
            pane_id: 42,
            session_id: "abc".into(),
            tool: "Edit".into(),
        },
    )
    .await;
    let snap = timeout(Duration::from_secs(2), handle.changed())
        .await
        .expect("not timed out")
        .expect("ok");
    assert_eq!(snap.get(42).unwrap().tool_call_count, 1);

    send(
        &sock,
        &HookEvent::SessionEnd {
            pane_id: 42,
            session_id: "abc".into(),
        },
    )
    .await;
    let snap = timeout(Duration::from_secs(2), handle.changed())
        .await
        .expect("not timed out")
        .expect("ok");
    assert!(snap.is_empty());
}
