use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tauri::{AppHandle, Emitter};

use super::blocks::{Block, BlockManager};
use super::session::{Session, SessionId};
use crate::agent::handlers;
use crate::agent::tools::ActiveTools;

#[derive(serde::Serialize, Clone)]
struct BlockStartIpcPayload {
    session_id: SessionId,
    id: String,
    command: String,
    started_at: u64,
    agent_id: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct BlockEndIpcPayload {
    session_id: SessionId,
    id: String,
    exit_code: i32,
    duration_ms: u64,
}

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

#[derive(serde::Serialize, Clone)]
pub struct TerminalTitlePayload {
    pub session_id: SessionId,
    pub title: String,
}

#[derive(serde::Serialize, Clone)]
pub struct TerminalCwdPayload {
    pub session_id: SessionId,
    pub cwd: String,
}

#[derive(serde::Serialize, Clone)]
pub struct TerminalHyperlinkPayload {
    pub session_id: SessionId,
    pub url: String,
    pub id: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct TerminalClipboardWritePayload {
    pub session_id: SessionId,
    /// Plain-text clipboard content (termwiz decodes base64 from the OSC 52 wire format).
    pub data: String,
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

    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    // Signal to agents that the terminal supports OSC 7777.
    cmd.env("TERM_OSC_7777", "1");

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

    let app_clone = app.clone();
    let sid = session_id.clone();

    // ActiveTools shared with the reader thread via Arc<Mutex<>>.
    // The session also holds a separate Mutex<ActiveTools> for IPC queries,
    // but the reader thread owns its own instance since it updates on every chunk.
    let tools_arc = std::sync::Arc::new(Mutex::new(ActiveTools::default()));
    let tools_for_thread = tools_arc.clone();

    // Blocks are shared via Arc so the session can serve `block_get` IPC queries.
    let blocks_arc: Arc<Mutex<Vec<Block>>> = Arc::new(Mutex::new(Vec::new()));
    let blocks_for_thread = blocks_arc.clone();

    std::thread::Builder::new()
        .name(format!("pty-reader-{sid}"))
        .spawn(move || {
            let mut buf = vec![0u8; 8192];
            // BlockManager tracks OSC 133 prompt marks for this session.
            let mut block_mgr = BlockManager::default();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];

                        // Extract OSC events before forwarding raw bytes.
                        let osc_events = crate::terminal::parser::extract_osc_events(chunk);

                        // Forward raw bytes to xterm.js unchanged.
                        let _ = app_clone.emit(
                            "terminal:output",
                            TerminalOutputPayload {
                                session_id: sid.clone(),
                                data: chunk.to_vec(),
                            },
                        );

                        // Dispatch OSC events.
                        let mut tools = tools_for_thread.lock().unwrap();
                        for event in osc_events {
                            use crate::agent::protocol::OscEvent;
                            match event {
                                OscEvent::Title(title) => {
                                    let _ = app_clone.emit(
                                        "terminal:title",
                                        TerminalTitlePayload {
                                            session_id: sid.clone(),
                                            title,
                                        },
                                    );
                                }
                                OscEvent::Cwd(cwd) => {
                                    let _ = app_clone.emit(
                                        "terminal:cwd",
                                        TerminalCwdPayload {
                                            session_id: sid.clone(),
                                            cwd,
                                        },
                                    );
                                }
                                OscEvent::Hyperlink { url, id } => {
                                    let _ = app_clone.emit(
                                        "terminal:hyperlink",
                                        TerminalHyperlinkPayload {
                                            session_id: sid.clone(),
                                            url,
                                            id,
                                        },
                                    );
                                }
                                OscEvent::ClipboardWrite(data) => {
                                    let _ = app_clone.emit(
                                        "terminal:clipboard-write",
                                        TerminalClipboardWritePayload {
                                            session_id: sid.clone(),
                                            data,
                                        },
                                    );
                                }
                                OscEvent::PromptMark { mark, exit_code, command } => {
                                    match mark {
                                        'A' => block_mgr.on_prompt_start(),
                                        'C' => {
                                            if let Some(data) = block_mgr.on_output_start(command) {
                                                // Store the new block for IPC queries.
                                                blocks_for_thread.lock().unwrap().push(Block {
                                                    id: data.id.clone(),
                                                    command: data.command.clone(),
                                                    started_at_ms: data.started_at_ms,
                                                    exit_code: None,
                                                    duration_ms: None,
                                                });
                                                let _ = app_clone.emit("block:start", BlockStartIpcPayload {
                                                    session_id: sid.clone(),
                                                    id: data.id,
                                                    command: data.command,
                                                    started_at: data.started_at_ms,
                                                    agent_id: None,
                                                });
                                            }
                                        }
                                        'D' => {
                                            if let Some(data) = block_mgr.on_command_end(exit_code.unwrap_or(0)) {
                                                // Update the stored block with exit code and duration.
                                                let mut blks = blocks_for_thread.lock().unwrap();
                                                if let Some(b) = blks.iter_mut().find(|b| b.id == data.id) {
                                                    b.exit_code = Some(data.exit_code);
                                                    b.duration_ms = Some(data.duration_ms);
                                                }
                                                drop(blks);
                                                let _ = app_clone.emit("block:end", BlockEndIpcPayload {
                                                    session_id: sid.clone(),
                                                    id: data.id,
                                                    exit_code: data.exit_code,
                                                    duration_ms: data.duration_ms,
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                OscEvent::Osc7777 { verb, payload } => {
                                    if let Some(inject) = handlers::dispatch(
                                        &verb,
                                        &payload,
                                        &sid,
                                        &mut tools,
                                        &app_clone,
                                    ) {
                                        let _ = app_clone.emit(
                                            "terminal:output",
                                            TerminalOutputPayload {
                                                session_id: sid.clone(),
                                                data: inject,
                                            },
                                        );
                                    }
                                }
                            }
                        }
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
        active_tools: Mutex::new(ActiveTools::default()),
        agent_name: Mutex::new(None),
        blocks: blocks_arc,
    })
}

fn default_shell() -> String {
    #[cfg(windows)]
    {
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

use std::io::Read;
