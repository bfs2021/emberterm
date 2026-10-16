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


/// Richer git status for the status bar: branch, dirty flag, and line diff counts.
#[derive(serde::Serialize, Clone)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub dirty: bool,
    pub insertions: u32,
    pub deletions: u32,
}

#[tauri::command]
pub async fn git_status(cwd: String) -> Result<GitStatus, String> {
    tokio::task::spawn_blocking(move || {
        // Normalise MINGW/Cygwin paths (/d/foo → D:/foo) so Windows git can
        // resolve the directory regardless of which shell reported the cwd.
        let cwd = normalise_cwd(&cwd);

        // Branch name
        let branch = git_run(&cwd, &["rev-parse", "--abbrev-ref", "HEAD"])
            .filter(|b| !b.is_empty() && b != "HEAD");

        if branch.is_none() {
            return GitStatus { branch: None, dirty: false, insertions: 0, deletions: 0 };
        }

        // Any uncommitted changes (staged or unstaged)?
        let dirty = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&cwd)
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

        // Line counts vs HEAD (staged + unstaged)
        let (insertions, deletions) = git_run(&cwd, &["diff", "HEAD", "--shortstat"])
            .map(|s| parse_shortstat(&s))
            .unwrap_or((0, 0));

        GitStatus { branch, dirty, insertions, deletions }
    })
    .await
    .map_err(|e| e.to_string())
}

/// Convert MINGW/Cygwin-style Unix paths to Windows paths so Rust's
/// `Command::current_dir` and git can resolve them.
/// `/d/foo/bar` → `D:/foo/bar`   (any single-letter drive prefix)
/// All other paths are returned unchanged.
fn normalise_cwd(cwd: &str) -> String {
    // Detect /X/... where X is a single ASCII letter (MINGW drive letter)
    if let Some(rest) = cwd.strip_prefix('/') {
        let mut chars = rest.chars();
        if let Some(drive) = chars.next() {
            if drive.is_ascii_alphabetic() {
                let after = chars.as_str();
                if after.is_empty() || after.starts_with('/') {
                    return format!("{}:{}", drive.to_ascii_uppercase(), after);
                }
            }
        }
    }
    cwd.to_string()
}

fn git_run(cwd: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Parse `git diff --shortstat` output: " 3 files changed, 12 insertions(+), 7 deletions(-)"
fn parse_shortstat(s: &str) -> (u32, u32) {
    fn extract(s: &str, marker: &str) -> u32 {
        s.find(marker)
            .and_then(|pos| s[..pos].trim_end().rsplit_once(' ').map(|(_, n)| n))
            .and_then(|n| n.trim_end_matches(',').parse().ok())
            .unwrap_or(0)
    }
    (extract(s, "insertion"), extract(s, "deletion"))
}

/// Run `<executable> --version` and extract a semver string like `v2.1.128`.
/// Returns `None` if the executable is not found or produces no version token.
/// Used by the frontend to fill in the agent version when OSC 7777 hello is absent.
#[tauri::command]
pub async fn query_agent_version(executable: String) -> Option<String> {
    tokio::task::spawn_blocking(move || {
        let out = std::process::Command::new(&executable)
            .arg("--version")
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(if out.stdout.is_empty() { &out.stderr } else { &out.stdout });
        extract_semver(&text)
    })
    .await
    .ok()?
}

/// Pull the first `X.Y.Z` or `vX.Y.Z` token out of version command output.
fn extract_semver(s: &str) -> Option<String> {
    for word in s.split_whitespace() {
        let stripped = word.trim_matches(|c: char| !c.is_ascii_digit() && c != 'v');
        let digits = stripped.strip_prefix('v').unwrap_or(stripped);
        let parts: Vec<&str> = digits.split('.').collect();
        if parts.len() >= 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
            return Some(format!("v{}", digits));
        }
    }
    None
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
