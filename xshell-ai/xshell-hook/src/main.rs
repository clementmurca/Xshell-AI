mod forward;

use forward::{forward, kind_from_arg};
use std::path::PathBuf;
use std::process::ExitCode;

fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("XSHELL_SOCK") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/tmp/xshell-ai-{uid}.sock"))
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1).unwrap_or_default();
    let Some(kind) = kind_from_arg(&arg) else {
        eprintln!(
            "xshell-hook: unknown kind '{arg}' (expected session-start|prompt-submit|pre-tool|post-tool|stop|session-end)"
        );
        return ExitCode::from(2);
    };

    match forward(kind, &socket_path()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xshell-hook: {e}");
            // Ne jamais faire échouer Claude Code sur une erreur de hook.
            ExitCode::SUCCESS
        }
    }
}
