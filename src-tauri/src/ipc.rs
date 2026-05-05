use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::terminal::session::SessionId;
use crate::terminal::pty;

#[tauri::command]
pub async fn session_create(
    profile_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SessionId, String> {
    let session_id = state.next_session_id();
    let session = pty::spawn_session(session_id.clone(), app)
        .map_err(|e| e.to_string())?;

    let _ = profile_id; // used in Phase 1 for profile-based shell selection

    state
        .sessions
        .lock()
        .unwrap()
        .insert(session_id.clone(), session);

    tracing::info!(session_id = %session_id, "session created");
    Ok(session_id)
}

#[tauri::command]
pub async fn session_write(
    id: SessionId,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&id).ok_or_else(|| format!("session {id} not found"))?;

    let mut writer = session.writer.lock().unwrap();
    use std::io::Write;
    writer.write_all(&data).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn session_resize(
    id: SessionId,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&id).ok_or_else(|| format!("session {id} not found"))?;

    session
        .master
        .resize(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn session_close(
    id: SessionId,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.sessions.lock().unwrap().remove(&id);
    tracing::info!(session_id = %id, "session closed");
    Ok(())
}
