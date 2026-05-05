use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tauri::{AppHandle, Emitter};

use super::blocks::{Block, BlockManager};
use super::events::{
    BlockAnnotateIpcPayload, BlockEndIpcPayload, BlockStartIpcPayload, SessionExitPayload,
    TerminalClipboardWritePayload, TerminalCwdPayload, TerminalHyperlinkPayload,
    TerminalOutputPayload, TerminalTitlePayload,
};
use super::session::{Session, SessionId};
use crate::agent::handlers;
use crate::agent::tools::ActiveTools;

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

    let writer: Arc<Mutex<Box<dyn std::io::Write + Send>>> = Arc::new(Mutex::new(
        pty_pair.master.take_writer().context("failed to take PTY writer")?,
    ));
    // Shared with the reader thread so agent.query replies can be written to
    // PTY stdin without going through the full IPC command path.
    let writer_for_thread = writer.clone();

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
            // Rate-limit OSC 7777 processing: max 1000 sequences per second.
            let mut osc7777_count: u32 = 0;
            let mut osc7777_window = std::time::Instant::now();
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
                                        // B mark: end-of-prompt, some shells embed command text here.
                                        'B' => block_mgr.on_command_start(command),
                                        'C' => {
                                            if let Some(data) = block_mgr.on_output_start(command) {
                                                // Store the new block for IPC queries.
                                                blocks_for_thread.lock().unwrap().push(Block {
                                                    id: data.id.clone(),
                                                    command: data.command.clone(),
                                                    started_at_ms: data.started_at_ms,
                                                    ..Default::default()
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
                                    // Enforce 1000 sequences/sec rate limit.
                                    if osc7777_window.elapsed().as_secs() >= 1 {
                                        osc7777_window = std::time::Instant::now();
                                        osc7777_count = 0;
                                    }
                                    osc7777_count += 1;
                                    if osc7777_count > 1000 {
                                        tracing::warn!(session_id = %sid, "OSC 7777 rate limit exceeded, dropping sequence");
                                    } else {
                                        match handlers::dispatch(&verb, &payload, &sid, &mut tools, &app_clone) {
                                            Some(handlers::DispatchEffect::TerminalOutput(data)) => {
                                                let _ = app_clone.emit("terminal:output", TerminalOutputPayload {
                                                    session_id: sid.clone(), data,
                                                });
                                            }
                                            Some(handlers::DispatchEffect::PtyInput(data)) => {
                                                let mut w = writer_for_thread.lock().unwrap();
                                                let _ = w.write_all(&data);
                                                let _ = w.flush();
                                            }
                                            Some(handlers::DispatchEffect::AnnotateBlock(p)) => {
                                                if let Some(block_id) = block_mgr.current_block_id() {
                                                    let block_id = block_id.clone();
                                                    // Update the stored block.
                                                    let mut blks = blocks_for_thread.lock().unwrap();
                                                    if let Some(b) = blks.iter_mut().find(|b| b.id == block_id) {
                                                        if p.agent_id.is_some() { b.agent_id = p.agent_id.clone(); }
                                                        if !p.tags.is_empty() { b.tags = p.tags.clone(); }
                                                        if p.fold.is_some() { b.fold = p.fold.clone(); }
                                                    }
                                                    drop(blks);
                                                    let _ = app_clone.emit("block:annotate", BlockAnnotateIpcPayload {
                                                        session_id: sid.clone(),
                                                        id: block_id,
                                                        agent_id: p.agent_id,
                                                        tags: p.tags,
                                                        fold: p.fold,
                                                    });
                                                }
                                            }
                                            None => {}
                                        }
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
        writer,
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

