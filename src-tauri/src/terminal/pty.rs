use std::sync::Mutex;

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tauri::{AppHandle, Emitter};

use super::session::{Session, SessionId};

#[derive(serde::Serialize, Clone)]
pub struct TerminalOutputPayload {
    pub session_id: SessionId,
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, Clone)]
pub struct SessionExitPayload {
    pub session_id: SessionId,
    pub code: i32,
}

/// Spawns a PTY session with the platform-default shell and starts the PTY
/// reader thread that forwards output bytes to the webview as Tauri events.
pub fn spawn_session(session_id: SessionId, app: AppHandle) -> Result<Session> {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("failed to open PTY")?;

    let shell = default_shell();
    let mut cmd = CommandBuilder::new(&shell);

    // Set a sane TERM value so shell prompts render correctly.
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    // Forward common env vars from the parent process.
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }

    let _child = pty_pair
        .slave
        .spawn_command(cmd)
        .context("failed to spawn shell")?;

    let mut reader = pty_pair
        .master
        .try_clone_reader()
        .context("failed to clone PTY reader")?;

    let writer = pty_pair
        .master
        .take_writer()
        .context("failed to take PTY writer")?;

    // Spawn a dedicated OS thread to read PTY output.
    // We use a real thread (not tokio) because the Read impl may block
    // indefinitely and tokio's blocking pool should not be occupied long-term.
    let app_clone = app.clone();
    let sid = session_id.clone();
    std::thread::Builder::new()
        .name(format!("pty-reader-{sid}"))
        .spawn(move || {
            let mut buf = vec![0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = app_clone.emit(
                            "terminal:output",
                            TerminalOutputPayload {
                                session_id: sid.clone(),
                                data: buf[..n].to_vec(),
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
            let _ = app_clone.emit(
                "terminal:exit",
                SessionExitPayload {
                    session_id: sid,
                    code: 0,
                },
            );
        })
        .context("failed to spawn PTY reader thread")?;

    Ok(Session {
        id: session_id,
        master: pty_pair.master,
        writer: Mutex::new(writer),
        title: shell,
        cwd: None,
    })
}

fn default_shell() -> String {
    #[cfg(windows)]
    {
        // Prefer PowerShell if available, fall back to cmd.exe.
        if which_shell("pwsh.exe") {
            "pwsh.exe".to_string()
        } else if which_shell("powershell.exe") {
            "powershell.exe".to_string()
        } else {
            "cmd.exe".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

#[cfg(windows)]
fn which_shell(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Read is used in the reader thread closure above.
use std::io::Read;
