use super::session::SessionId;

/// IPC event payload structs emitted as Tauri events from the PTY reader thread.
/// All fields use snake_case — Tauri/serde serializes them to camelCase for the frontend.

#[derive(serde::Serialize, Clone)]
pub struct SessionExitPayload {
    pub session_id: SessionId,
    pub code: i32,
}
