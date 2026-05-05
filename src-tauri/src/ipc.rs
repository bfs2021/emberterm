use tauri::{AppHandle, State};

use crate::config::Config;
use crate::profile::Profile;
use crate::state::AppState;
use crate::terminal::session::SessionId;
use crate::terminal::pty;

// ── Session commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn session_create(
    profile_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SessionId, String> {
    let session_id = state.next_session_id();
    let _ = profile_id; // Phase 1+: use profile.shell
    let session = pty::spawn_session(session_id.clone(), app)
        .map_err(|e| e.to_string())?;
    state.sessions.lock().unwrap().insert(session_id.clone(), session);
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
        .resize(portable_pty::PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
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

// ── Config commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn config_get(state: State<'_, AppState>) -> Result<Config, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub async fn config_update(
    patch: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<Config, String> {
    let mut cfg = state.config.lock().unwrap();
    let mut current = serde_json::to_value(cfg.clone()).map_err(|e| e.to_string())?;
    json_merge(&mut current, patch);
    *cfg = serde_json::from_value(current).map_err(|e| e.to_string())?;
    cfg.save().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

// ── Profile commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn profile_list(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    Ok(state.profiles.lock().unwrap().clone())
}

#[tauri::command]
pub async fn profile_upsert(
    profile: Profile,
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::profile::save_profile(&profile).map_err(|e| e.to_string())?;
    let mut profiles = state.profiles.lock().unwrap();
    match profiles.iter_mut().find(|p| p.id == profile.id) {
        Some(existing) => *existing = profile,
        None => profiles.push(profile),
    }
    Ok(())
}

// ── Agent commands ───────────────────────────────────────────────────────────

/// Write an OSC 7777 message back into the agent's PTY input stream.
/// `verb` is typically "reply" or "event"; `payload` is a JSON string.
#[tauri::command]
pub async fn agent_respond(
    session_id: SessionId,
    verb: String,
    payload: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Format: ESC ] 7777 ; <verb> ; <payload> ESC \
    let sequence = format!("\x1b]7777;{};{}\x1b\\", verb, payload);
    let sessions = state.sessions.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session {session_id} not found"))?;
    let mut writer = session.writer.lock().unwrap();
    use std::io::Write;
    writer.write_all(sequence.as_bytes()).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Shell out to git to get the current branch name for the given directory.
/// Returns `None` if not a git repository or git is not installed.
#[tauri::command]
pub async fn git_branch(cwd: String) -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(move || -> Option<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&cwd)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() || branch == "HEAD" {
                None
            } else {
                Some(branch)
            }
        } else {
            None
        }
    })
    .await
    .map_err(|e| e.to_string())
}

/// Returns true when no `config.toml` exists yet — used to show the welcome screen.
#[tauri::command]
pub async fn config_is_first_run() -> bool {
    match crate::config::config_path() {
        Some(path) => !path.exists(),
        None => false,
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn json_merge(base: &mut serde_json::Value, patch: serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                json_merge(b.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (base, patch) => *base = patch,
    }
}
