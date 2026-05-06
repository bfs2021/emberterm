use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tauri::{AppHandle, Emitter};

use super::blocks::{Block, BlockManager, BlockStore};
use super::events::{
    BlockAnnotateIpcPayload, BlockEndIpcPayload, BlockStartIpcPayload, KittyKeyboardPayload,
    SessionExitPayload, TerminalClipboardWritePayload, TerminalCwdPayload,
    TerminalHyperlinkPayload, TerminalOutputPayload, TerminalTitlePayload,
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
    let blocks_arc: Arc<Mutex<BlockStore>> = Arc::new(Mutex::new(BlockStore::default()));
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
            // DEC 2026 synchronized output: buffer chunks between begin/end markers.
            let mut sync_buf: Vec<u8> = Vec::new();
            let mut in_sync = false;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];

                        // Extract OSC events before forwarding raw bytes.
                        let osc_events = crate::terminal::parser::extract_osc_events(chunk);

                        // Detect Kitty keyboard push/pop sequences.
                        if let Some(flags) = scan_kitty_keyboard(chunk) {
                            let _ = app_clone.emit("terminal:kitty-keyboard", KittyKeyboardPayload {
                                session_id: sid.clone(),
                                flags,
                            });
                        }

                        // Kitty graphics: raw bytes are forwarded to xterm.js unchanged;
                        // the @xterm/addon-image addon handles rendering. Log at debug level
                        // so developers can confirm graphics sequences are flowing through.
                        if has_kitty_graphics(chunk) {
                            tracing::debug!(session_id = %sid, "Kitty graphics APC detected in PTY output");
                        }

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

                        let _ = app_clone.emit(
                            "terminal:output",
                            TerminalOutputPayload {
                                session_id: sid.clone(),
                                data: to_emit,
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
                                                // Store the new block; O(1) via BlockStore.push().
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
                                                // O(1) lookup via BlockStore.get_mut().
                                                if let Some(b) = blocks_for_thread.lock().unwrap().get_mut(&data.id) {
                                                    b.exit_code = Some(data.exit_code);
                                                    b.duration_ms = Some(data.duration_ms);
                                                }
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
                                                    // O(1) lookup via BlockStore.get_mut().
                                                    if let Some(b) = blocks_for_thread.lock().unwrap().get_mut(&block_id) {
                                                        if p.agent_id.is_some() { b.agent_id = p.agent_id.clone(); }
                                                        if !p.tags.is_empty() { b.tags = p.tags.clone(); }
                                                        if p.fold.is_some() { b.fold = p.fold.clone(); }
                                                    }
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

/// Scans a PTY output chunk for Kitty keyboard protocol push/pop sequences.
///
/// Returns `Some(flags)` when:
/// - A push sequence `ESC [ > N u` is found (flags = N, may be 0)
/// - A pop  sequence `ESC [ < u`   is found (flags = 0 signals "popped")
///
/// Returns `None` if no Kitty sequence is present in this chunk.
///
/// Limitation: sequences split across chunk boundaries are not detected.
/// This is acceptable because applications write these as atomic strings
/// well under the 8 KiB read buffer.
pub(super) fn scan_kitty_keyboard(bytes: &[u8]) -> Option<u32> {
    let s = std::str::from_utf8(bytes).ok()?;
    let mut iter = s.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        if c != '\x1b' {
            continue;
        }
        let rest = &s[i + 1..];
        // Pop: ESC [ < u
        if rest.starts_with("[<u") {
            return Some(0);
        }
        // Push: ESC [ > <digits> u
        if let Some(suffix) = rest.strip_prefix("[>") {
            let end = suffix.find(|c: char| !c.is_ascii_digit()).unwrap_or(suffix.len());
            if suffix[end..].starts_with('u') {
                let flags: u32 = suffix[..end].parse().unwrap_or(0);
                return Some(flags);
            }
        }
    }
    None
}

/// Returns true if the chunk contains the start of a Kitty graphics APC sequence
/// (`ESC _ G ...`). Used only for capability detection and debug logging; the
/// raw bytes are forwarded to xterm.js unchanged for rendering via the image addon.
pub(super) fn has_kitty_graphics(bytes: &[u8]) -> bool {
    // Kitty graphics: ESC _ G  (APC with 'G' introducer)
    bytes.windows(3).any(|w| w == b"\x1b_G")
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

#[cfg(test)]
mod tests {
    use super::scan_kitty_keyboard;

    #[test]
    fn kitty_push_flags_1() {
        // ESC [ > 1 u  →  flags = 1
        assert_eq!(scan_kitty_keyboard(b"\x1b[>1u"), Some(1));
    }

    #[test]
    fn kitty_push_flags_31() {
        // ESC [ > 31 u  →  flags = 31
        assert_eq!(scan_kitty_keyboard(b"\x1b[>31u"), Some(31));
    }

    #[test]
    fn kitty_push_flags_0() {
        // ESC [ > 0 u  →  flags = 0 (push with no flags — still a push)
        assert_eq!(scan_kitty_keyboard(b"\x1b[>0u"), Some(0));
    }

    #[test]
    fn kitty_pop() {
        // ESC [ < u  →  flags = 0 (pop)
        assert_eq!(scan_kitty_keyboard(b"\x1b[<u"), Some(0));
    }

    #[test]
    fn kitty_embedded_in_longer_chunk() {
        // Sequence embedded inside printable bytes
        let bytes = b"hello\x1b[>3uworld";
        assert_eq!(scan_kitty_keyboard(bytes), Some(3));
    }

    #[test]
    fn kitty_no_sequence() {
        // Normal PTY output with no Kitty sequences
        assert_eq!(scan_kitty_keyboard(b"ls -la\r\n"), None);
    }

    #[test]
    fn kitty_csi_arrow_not_matched() {
        // ESC [ A (cursor up) must not be detected as a Kitty sequence
        assert_eq!(scan_kitty_keyboard(b"\x1b[A"), None);
    }

    // ── Kitty graphics ────────────────────────────────────────────────────────

    use super::has_kitty_graphics;

    #[test]
    fn kitty_graphics_apc_detected() {
        let data = b"\x1b_Ga=T,f=32,s=1,v=1,m=0,x=0,y=0;AAAA\x1b\\";
        assert!(has_kitty_graphics(data));
    }

    #[test]
    fn kitty_graphics_not_present() {
        assert!(!has_kitty_graphics(b"ls -la\r\n"));
        assert!(!has_kitty_graphics(b"\x1b[>1u"));
    }

    // ── DEC 2026 synchronized output ──────────────────────────────────────────

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

