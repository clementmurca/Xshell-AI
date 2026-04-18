use crate::error::{AgentError, Result};
use crate::events::parse_event;
use crate::state::AgentStore;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

/// Handle clonable pour consommer les snapshots du store.
#[derive(Clone)]
pub struct AgentStoreHandle {
    rx: watch::Receiver<AgentStore>,
}

impl AgentStoreHandle {
    /// Renvoie un snapshot courant (cloné).
    pub fn snapshot(&self) -> AgentStore {
        self.rx.borrow().clone()
    }

    /// Attend la prochaine mise à jour, retourne le snapshot post-update.
    pub async fn changed(&mut self) -> Result<AgentStore> {
        self.rx.changed().await.map_err(|_| AgentError::Io {
            path: PathBuf::from("<watch-closed>"),
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "watch channel closed"),
        })?;
        Ok(self.rx.borrow().clone())
    }
}

/// Démarre un listener sur `socket_path` dans un task tokio.
/// Retourne un `AgentStoreHandle` et un `JoinHandle` pour la boucle.
pub async fn start_server(
    socket_path: &Path,
) -> Result<(AgentStoreHandle, JoinHandle<()>)> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path).map_err(|source| AgentError::Io {
            path: socket_path.to_path_buf(),
            source,
        })?;
    }

    let listener = UnixListener::bind(socket_path).map_err(|source| AgentError::Io {
        path: socket_path.to_path_buf(),
        source,
    })?;

    let store = Arc::new(Mutex::new(AgentStore::new()));
    let (tx, rx) = watch::channel(AgentStore::new());

    let store_cloned = Arc::clone(&store);
    let handle = tokio::spawn(async move {
        loop {
            let (stream, _addr) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };
            let store = Arc::clone(&store_cloned);
            let tx = tx.clone();
            tokio::spawn(async move {
                handle_connection(stream, store, tx).await;
            });
        }
    });

    Ok((AgentStoreHandle { rx }, handle))
}

async fn handle_connection(
    stream: UnixStream,
    store: Arc<Mutex<AgentStore>>,
    tx: watch::Sender<AgentStore>,
) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let event = match parse_event(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let mut guard = store.lock().await;
        let changed = guard.apply(event);
        if changed {
            let _ = tx.send(guard.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::HookEvent;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{timeout, Duration};

    async fn write_event(path: &Path, ev: &HookEvent) {
        let mut stream = UnixStream::connect(path).await.expect("connect");
        let mut line = serde_json::to_string(ev).expect("serialize");
        line.push('\n');
        stream.write_all(line.as_bytes()).await.expect("write");
        stream.shutdown().await.ok();
    }

    #[tokio::test]
    async fn session_start_populates_store() {
        let dir = tempdir().unwrap();
        let sock = dir.path().join("t.sock");
        let (mut handle, _jh) = start_server(&sock).await.expect("start");

        write_event(
            &sock,
            &HookEvent::SessionStart {
                pane_id: 1,
                session_id: "s".into(),
                cwd: PathBuf::from("/tmp"),
                model: None,
            },
        )
        .await;

        let snap = timeout(Duration::from_secs(2), handle.changed())
            .await
            .expect("not timed out")
            .expect("ok");
        assert_eq!(snap.len(), 1);
        assert!(snap.get(1).is_some());
    }

    #[tokio::test]
    async fn garbage_lines_do_not_crash() {
        let dir = tempdir().unwrap();
        let sock = dir.path().join("t.sock");
        let (handle, _jh) = start_server(&sock).await.expect("start");

        let mut stream = UnixStream::connect(&sock).await.expect("connect");
        stream.write_all(b"not json\n").await.expect("write");
        stream.shutdown().await.ok();

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(handle.snapshot().is_empty());
    }

    #[tokio::test]
    async fn multiple_events_accumulate() {
        let dir = tempdir().unwrap();
        let sock = dir.path().join("t.sock");
        let (mut handle, _jh) = start_server(&sock).await.expect("start");

        for pane in [1u64, 2, 3] {
            write_event(
                &sock,
                &HookEvent::SessionStart {
                    pane_id: pane,
                    session_id: format!("s{pane}"),
                    cwd: PathBuf::from("/tmp"),
                    model: None,
                },
            )
            .await;
            timeout(Duration::from_secs(2), handle.changed())
                .await
                .expect("not timed out")
                .expect("ok");
        }
        assert_eq!(handle.snapshot().len(), 3);
    }
}
