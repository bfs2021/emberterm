use super::session::SessionId;

/// IPC event payload structs emitted as Tauri events from the PTY reader thread.
/// All fields use snake_case — Tauri/serde serializes them to camelCase for the frontend.

/// Emitted when an application running in the PTY pushes or pops Kitty keyboard mode.
/// `flags` is the Kitty enhancement flags bitmask (0 = pop/disabled).
/// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>
#[derive(serde::Serialize, Clone)]
pub struct KittyKeyboardPayload {
    pub session_id: SessionId,
    /// Active Kitty keyboard flags, or 0 if mode was popped.
    pub flags: u32,
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

#[derive(serde::Serialize, Clone)]
pub(super) struct BlockAnnotateIpcPayload {
    pub session_id: SessionId,
    pub id: String,
    pub agent_id: Option<String>,
    pub tags: Vec<String>,
    pub fold: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub(super) struct BlockStartIpcPayload {
    pub session_id: SessionId,
    pub id: String,
    pub command: String,
    pub started_at: u64,
    pub agent_id: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub(super) struct BlockEndIpcPayload {
    pub session_id: SessionId,
    pub id: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}
