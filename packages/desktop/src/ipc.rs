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

    // Resolve which profile to use:
    //   1. The explicitly requested profile_id, if provided.
    //   2. The config's default_profile setting.
    //   3. The first profile in the list.
    //   4. An inline default (no-clone fallback).
    let profile: Option<crate::profile::Profile> = {
        let profiles = state.profiles.lock().unwrap();
        let cfg = state.config.lock().unwrap();
        if let Some(ref id) = profile_id {
            profiles.iter().find(|p| p.id == *id).cloned()
        } else {
            let default_id = cfg.behavior.default_profile.clone();
            profiles
                .iter()
                .find(|p| p.id == default_id)
                .or_else(|| profiles.first())
                .cloned()
        }
    };

    let session = pty::spawn_session(session_id.clone(), profile.as_ref(), app)
        .map_err(|e| e.to_string())?;
    state.sessions.lock().unwrap().insert(session_id.clone(), session);
    tracing::info!(
        session_id = %session_id,
        profile = profile.as_ref().map(|p| p.id.as_str()).unwrap_or("default"),
        "session created"
    );
    Ok(session_id)
}

#[tauri::command]
pub async fn session_write(
    id: SessionId,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Clone Arc while holding sessions lock, then release before locking writer.
    let writer_arc = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&id)
            .ok_or_else(|| format!("session {id} not found"))?
            .writer.clone()
    };
    let mut writer = writer_arc.lock().unwrap();
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

#[tauri::command]
pub async fn profile_delete(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::profile::delete_profile(&id).map_err(|e| e.to_string())?;
    let mut profiles = state.profiles.lock().unwrap();
    profiles.retain(|p| p.id != id);
    Ok(())
}

// ── Block commands ───────────────────────────────────────────────────────────

/// Returns all blocks recorded for a session, ordered by start time.
/// Useful for the block UI to restore state after reconnecting.
#[tauri::command]
pub async fn block_get(
    session_id: SessionId,
    state: State<'_, AppState>,
) -> Result<Vec<crate::terminal::blocks::Block>, String> {
    // Clone the Arc while holding the sessions lock, then release it before
    // locking blocks — avoids holding two locks simultaneously.
    let blocks_arc = {
        let sessions = state.sessions.lock().unwrap();
        sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {session_id} not found"))?
            .blocks
            .clone()
    };
    let blocks = blocks_arc.lock().unwrap().all().to_vec();
    Ok(blocks)
}

/// Searches blocks for a session by command text (case-insensitive substring match).
/// An empty query returns all blocks.
#[tauri::command]
pub async fn block_search(
    session_id: SessionId,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::terminal::blocks::Block>, String> {
    let blocks_arc = {
        let sessions = state.sessions.lock().unwrap();
        sessions
            .get(&session_id)
            .ok_or_else(|| format!("session {session_id} not found"))?
            .blocks
            .clone()
    };
    let store = blocks_arc.lock().unwrap();
    let q = query.to_lowercase();
    let results = if q.is_empty() {
        store.all().to_vec()
    } else {
        store.all().iter().filter(|b| b.command.to_lowercase().contains(&q)).cloned().collect()
    };
    Ok(results)
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
    // Clone Arc while holding sessions lock, then release before locking writer.
    let writer_arc = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| format!("session {session_id} not found"))?
            .writer.clone()
    };
    let mut writer = writer_arc.lock().unwrap();
    use std::io::Write;
    writer.write_all(sequence.as_bytes()).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Toggle quake mode — hide if visible, show/reposition if hidden.
/// Called by the in-window keyboard shortcut (Ctrl+Backquote) from the frontend.
/// The OS-level global shortcut is registered separately via `tauri-plugin-global-shortcut`.
#[tauri::command]
pub async fn quake_toggle(app: AppHandle) {
    crate::quake::toggle(&app);
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

// ── Built-in profile detection ───────────────────────────────────────────────

/// A shell candidate that EmberTerm knows about but hasn't been saved by the user yet.
#[derive(serde::Serialize, Clone)]
pub struct BuiltinProfileSpec {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub shell: String,
    pub args: Vec<String>,
    pub description: String,
}

/// Return `true` when `name` is found on PATH (uses `where` on Windows, `which` elsewhere).
fn probe(name: &str) -> bool {
    #[cfg(windows)]
    let cmd = "where";
    #[cfg(not(windows))]
    let cmd = "which";

    std::process::Command::new(cmd)
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Scan the system for well-known shells and return those that are available.
/// The list is ordered: most-common first. Already-saved user profiles are
/// filtered out on the frontend by comparing shell paths.
#[tauri::command]
pub async fn detect_builtin_profiles() -> Vec<BuiltinProfileSpec> {
    tokio::task::spawn_blocking(|| {
        let mut specs: Vec<BuiltinProfileSpec> = Vec::new();

        #[cfg(windows)]
        {
            // cmd.exe — always present
            if std::path::Path::new("C:/Windows/System32/cmd.exe").exists() {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-cmd".into(),
                    name: "Command Prompt".into(),
                    icon: "terminal".into(),
                    shell: "cmd.exe".into(),
                    args: vec![],
                    description: "Windows Command Prompt".into(),
                });
            }

            // Windows PowerShell 5 — ships with every Windows 10/11
            if std::path::Path::new(
                "C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe",
            )
            .exists()
                || probe("powershell.exe")
            {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-powershell5".into(),
                    name: "Windows PowerShell".into(),
                    icon: "pwsh".into(),
                    shell: "powershell.exe".into(),
                    args: vec!["-NoLogo".into()],
                    description: "PowerShell 5 (built into Windows)".into(),
                });
            }

            // PowerShell 7+ (pwsh)
            if probe("pwsh.exe") {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-pwsh".into(),
                    name: "PowerShell".into(),
                    icon: "pwsh".into(),
                    shell: "pwsh.exe".into(),
                    args: vec!["-NoLogo".into()],
                    description: "PowerShell 7+".into(),
                });
            }

            // Git Bash — check standard installation paths
            let local_app = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let local_git = format!("{}/Programs/Git/bin/bash.exe", local_app.replace('\\', "/"));
            let git_bash_candidates = [
                "C:/Program Files/Git/bin/bash.exe",
                "C:/Program Files (x86)/Git/bin/bash.exe",
                local_git.as_str(),
            ];
            if let Some(path) = git_bash_candidates
                .iter()
                .find(|p| std::path::Path::new(p).exists())
            {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-gitbash".into(),
                    name: "Git Bash".into(),
                    icon: "bash".into(),
                    shell: path.to_string(),
                    args: vec!["--login".into(), "-i".into()],
                    description: "Git for Windows Bash shell".into(),
                });
            }

            // WSL — wsl.exe lives in System32 on supported Windows builds
            if std::path::Path::new("C:/Windows/System32/wsl.exe").exists()
                || probe("wsl.exe")
            {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-wsl".into(),
                    name: "WSL".into(),
                    icon: "bash".into(),
                    shell: "wsl.exe".into(),
                    args: vec![],
                    description: "Windows Subsystem for Linux".into(),
                });
            }

            // Docker
            if probe("docker.exe") || probe("docker") {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-docker".into(),
                    name: "Docker".into(),
                    icon: "docker".into(),
                    shell: "docker".into(),
                    args: vec![
                        "run".into(),
                        "-it".into(),
                        "--rm".into(),
                        "ubuntu".into(),
                        "/bin/bash".into(),
                    ],
                    description: "Docker container (ubuntu)".into(),
                });
            }
        }

        #[cfg(not(windows))]
        {
            let shell_candidates: &[(&str, &str, &str, &str, &[&str], &str)] = &[
                ("/bin/zsh",          "Zsh",  "builtin-zsh",  "bash", &[], "Z Shell"),
                ("/bin/bash",         "Bash", "builtin-bash", "bash", &[], "Bourne Again Shell"),
                ("/usr/bin/fish",     "Fish", "builtin-fish", "bash", &[], "Friendly Interactive Shell"),
                ("/usr/local/bin/fish","Fish","builtin-fish", "bash", &[], "Friendly Interactive Shell"),
                ("/bin/sh",           "sh",   "builtin-sh",   "terminal", &[], "POSIX Shell"),
            ];
            let mut seen_fish = false;
            for (path, name, id, icon, args, desc) in shell_candidates {
                if *id == "builtin-fish" {
                    if seen_fish { continue; }
                    seen_fish = true;
                }
                if std::path::Path::new(path).exists() {
                    specs.push(BuiltinProfileSpec {
                        id: id.to_string(),
                        name: name.to_string(),
                        icon: icon.to_string(),
                        shell: path.to_string(),
                        args: args.iter().map(|s| s.to_string()).collect(),
                        description: desc.to_string(),
                    });
                }
            }
            if probe("docker") {
                specs.push(BuiltinProfileSpec {
                    id: "builtin-docker".into(),
                    name: "Docker".into(),
                    icon: "docker".into(),
                    shell: "docker".into(),
                    args: vec![
                        "run".into(),
                        "-it".into(),
                        "--rm".into(),
                        "ubuntu".into(),
                        "/bin/bash".into(),
                    ],
                    description: "Docker container (ubuntu)".into(),
                });
            }
        }

        specs
    })
    .await
    .unwrap_or_default()
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
