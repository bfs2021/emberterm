use std::io::Read;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter};

use super::events::SessionExitPayload;
use super::session::{Session, SessionId};

/// Spawns a PTY session using `profile` (or the platform default shell when
/// `None`) and starts the PTY reader thread that forwards output bytes to the
/// webview as Tauri events.
pub fn spawn_session(
    session_id: SessionId,
    profile: Option<&crate::profile::Profile>,
    on_output: Channel,
    app: AppHandle,
) -> Result<Session> {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("failed to open PTY")?;

    let shell = profile
        .map(|p| p.shell.clone())
        .unwrap_or_else(super::which::default_shell);

    let mut cmd = CommandBuilder::new(&shell);

    // Profile-defined arguments (e.g. ["-NoLogo"] for pwsh, ["--login", "-i"] for git bash)
    if let Some(p) = profile {
        for arg in &p.args {
            cmd.arg(arg);
        }
        // Set working directory
        cmd.cwd(p.resolved_cwd());
        // Per-profile environment overrides
        for (k, v) in &p.env {
            cmd.env(k, v);
        }
    }

    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }

    // Shell integration: emit OSC 7 (cwd) before every prompt for bash/zsh/sh.
    // This enables the git status display and cwd tracking in the status bar.
    // The env var is only set when no PROMPT_COMMAND is already present so we
    // don't clobber a user's existing config.  If the user's .bashrc sets their
    // own PROMPT_COMMAND they can append the sequence themselves.
    if is_bash_like(&shell) && std::env::var("PROMPT_COMMAND").is_err() {
        cmd.env(
            "PROMPT_COMMAND",
            r#"printf '\033]7;file://%s\033\\' "$PWD""#,
        );
    }

    let _child = pty_pair
        .slave
        .spawn_command(cmd)
        .context("failed to spawn shell")?;

    let mut reader = pty_pair
        .master
        .try_clone_reader()
        .context("failed to clone PTY reader")?;

    let writer: Arc<Mutex<Box<dyn std::io::Write + Send>>> = Arc::new(Mutex::new(
        pty_pair
            .master
            .take_writer()
            .context("failed to take PTY writer")?,
    ));

    let app_clone = app.clone();
    let sid = session_id.clone();

    std::thread::Builder::new()
        .name(format!("pty-reader-{sid}"))
        .spawn(move || {
            let exit_app = app_clone.clone();
            let exit_sid = sid.clone();
            let _exit_guard = OnDrop(move || {
                let _ = exit_app.emit(
                    "terminal:exit",
                    SessionExitPayload {
                        session_id: exit_sid.clone(),
                        code: 0,
                    },
                );
            });
            let mut buf = vec![0u8; 8192];
            // DEC 2026 synchronized output: buffer chunks between begin/end markers.
            let mut sync_buf: Vec<u8> = Vec::new();
            let mut in_sync = false;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];

                        // DEC 2026: detect begin/end synchronized output markers.
                        // \x1b[?2026h = begin, \x1b[?2026l = end.
                        // Buffer chunks during sync to coalesce into a single IPC event.
                        let had_sync_end = detect_sync(chunk, &mut in_sync, &mut sync_buf);

                        // Forward bytes to xterm.js: either the buffered+flushed sync data,
                        // or the raw chunk (when not in sync mode).
                        let to_emit: Vec<u8> = if had_sync_end {
                            // sync_buf was flushed by detect_sync; take it.
                            std::mem::take(&mut sync_buf)
                        } else if in_sync {
                            // Still accumulating — don't emit yet.
                            continue;
                        } else {
                            chunk.to_vec()
                        };

                        if let Err(e) = on_output.send(InvokeResponseBody::Raw(to_emit)) {
                            tracing::debug!(session_id = %sid, error = %e, "output channel closed; stopping reader");
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .context("failed to spawn PTY reader thread")?;

    Ok(Session {
        master: pty_pair.master,
        writer,
    })
}

/// Runs a closure on drop — including during a panic unwind — so the
/// frontend always learns that a pane's reader thread has stopped.
pub(crate) struct OnDrop<F: FnMut()>(pub F);

impl<F: FnMut()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

/// DEC 2026 synchronized output — update sync state based on `chunk`.
///
/// - Detects `ESC [ ? 2026 h` (begin sync) and `ESC [ ? 2026 l` (end sync).
/// - While in sync mode, appends incoming bytes to `sync_buf` instead of emitting.
/// - Returns `true` when an end-sync marker is found; at that point `sync_buf`
///   holds the complete coalesced frame and the caller should emit it.
pub(super) fn detect_sync(chunk: &[u8], in_sync: &mut bool, sync_buf: &mut Vec<u8>) -> bool {
    const BEGIN: &[u8] = b"\x1b[?2026h";
    const END: &[u8] = b"\x1b[?2026l";

    let has_begin = chunk.windows(BEGIN.len()).any(|w| w == BEGIN);
    let has_end = chunk.windows(END.len()).any(|w| w == END);

    if has_begin && !*in_sync {
        *in_sync = true;
    }

    if *in_sync {
        sync_buf.extend_from_slice(chunk);
    }

    if has_end && *in_sync {
        *in_sync = false;
        return true; // caller should emit sync_buf
    }

    false
}

/// Returns true for bash, zsh, sh, or any variant ending in those names
/// (e.g. "git bash", "bash.exe").  Used to decide whether to inject OSC 7
/// emission via PROMPT_COMMAND.
fn is_bash_like(shell: &str) -> bool {
    let name = shell
        .split(['/', '\\'])
        .next_back()
        .unwrap_or(shell)
        .to_lowercase();
    let stem = name.trim_end_matches(".exe");
    matches!(stem, "bash" | "zsh" | "sh" | "ash" | "dash")
}

#[cfg(test)]
mod tests {
    // ── DEC 2026 synchronized output ──────────────────────────────────────────

    #[test]
    fn on_drop_runs_during_panic() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        let ran = Arc::new(AtomicBool::new(false));
        let flag = ran.clone();
        let result = std::panic::catch_unwind(move || {
            let _g = super::OnDrop(move || flag.store(true, Ordering::SeqCst));
            panic!("boom");
        });
        assert!(result.is_err());
        assert!(ran.load(Ordering::SeqCst));
    }

    use super::detect_sync;

    #[test]
    fn dec2026_begin_and_end_in_same_chunk_returns_true() {
        let mut in_sync = false;
        let mut buf = Vec::new();
        // One chunk contains both begin and end
        let chunk = b"\x1b[?2026hsome output\x1b[?2026l";
        assert!(detect_sync(chunk, &mut in_sync, &mut buf));
        assert!(!in_sync);
        assert_eq!(buf, chunk);
    }

    #[test]
    fn dec2026_begin_chunk_does_not_emit() {
        let mut in_sync = false;
        let mut buf = Vec::new();
        // First chunk: begin only
        let chunk1 = b"\x1b[?2026hpart1";
        assert!(!detect_sync(chunk1, &mut in_sync, &mut buf));
        assert!(in_sync);
        assert_eq!(&buf[..chunk1.len()], chunk1.as_ref());
    }

    #[test]
    fn dec2026_end_chunk_coalesces_and_returns_true() {
        let mut in_sync = false;
        let mut buf = Vec::new();
        let chunk1 = b"\x1b[?2026hpart1";
        let chunk2 = b"part2\x1b[?2026l";
        detect_sync(chunk1, &mut in_sync, &mut buf);
        assert!(detect_sync(chunk2, &mut in_sync, &mut buf));
        assert!(!in_sync);
        let expected: Vec<u8> = [chunk1.as_ref(), chunk2.as_ref()].concat();
        assert_eq!(buf, expected);
    }

    #[test]
    fn dec2026_no_markers_plain_passthrough() {
        let mut in_sync = false;
        let mut buf = Vec::new();
        assert!(!detect_sync(b"plain output", &mut in_sync, &mut buf));
        assert!(!in_sync);
        assert!(buf.is_empty());
    }
}
