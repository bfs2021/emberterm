use std::path::{Path, PathBuf};

use base64::Engine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Wsl,
    Pwsh,
    Other,
}

pub fn shell_kind(shell: &str) -> ShellKind {
    let name = shell
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(shell)
        .to_lowercase();
    match name.trim_end_matches(".exe") {
        "bash" | "sh" | "zsh" => ShellKind::Bash,
        "wsl" => ShellKind::Wsl,
        "pwsh" | "powershell" => ShellKind::Pwsh,
        _ => ShellKind::Other,
    }
}

pub struct HookInput<'a> {
    pub kind: ShellKind,
    pub profile_args: &'a [String],
    pub profile_cwd: PathBuf,
    /// Last folder this pane reported via OSC 7, in the shell's own format.
    pub restore_cwd: Option<&'a str>,
    pub inherited_prompt_command: bool,
    pub inherited_wslenv: Option<&'a str>,
    pub dir_exists: &'a dyn Fn(&Path) -> bool,
}

pub struct SpawnPlan {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: PathBuf,
}

/// Emits OSC 7 for `$PWD`; avoids `pwd -W`, whose fork is slow under MSYS. `%`, ESC and
/// BEL are percent-encoded so a folder name cannot end the sequence early or be
/// mis-decoded, and `$?` is preserved for anything the user appends to PROMPT_COMMAND.
const BASH_OSC7: &str = r#"__ember_osc7() { local s=$? p=${PWD//%/%25}; p=${p//$'\e'/%1B}; p=${p//$'\a'/%07}; printf '\033]7;file://localhost%s\033\\' "$p"; return $s; }; __ember_osc7"#;
/// Runs once before the first WSL prompt; a missing folder is a silent no-op.
const WSL_RESTORE: &str =
    r#"[ -n "$EMBER_CWD" ] && { cd -- "$EMBER_CWD" 2>/dev/null; unset EMBER_CWD; }"#;
/// Runs after $PROFILE so prompt themes (oh-my-posh, starship) are wrapped, not replaced.
/// `[char]27` instead of `` `e `` keeps Windows PowerShell 5.1 working. The original prompt runs
/// first so `$?` is still the user's last command status, and the path is percent-encoded so the
/// OSC stays ASCII whatever the console code page is.
const PWSH_HOOK: &str = r#"$global:__ember_prompt = $function:prompt
function global:prompt {
  $o = & $global:__ember_prompt
  $p = $executionContext.SessionState.Path.CurrentFileSystemLocation.ProviderPath
  $e = ($p -split '\\' | ForEach-Object { [uri]::EscapeDataString($_) }) -join '/'
  [Console]::Write("$([char]27)]7;file://localhost/$e$([char]27)\")
  $o
}"#;

pub fn prepare(input: &HookInput) -> SpawnPlan {
    let mut plan = SpawnPlan {
        args: input.profile_args.to_vec(),
        env: Vec::new(),
        cwd: input.profile_cwd.clone(),
    };
    match input.kind {
        ShellKind::Bash => {
            if !input.inherited_prompt_command {
                plan.env.push(("PROMPT_COMMAND".into(), BASH_OSC7.into()));
            }
            let dir = input.restore_cwd.and_then(|raw| {
                if cfg!(windows) {
                    msys_to_windows(raw)
                } else {
                    Some(raw.to_string())
                }
            });
            apply_restore_dir(&mut plan, dir, input.dir_exists);
        }
        ShellKind::Wsl => {
            plan.env.push((
                "PROMPT_COMMAND".into(),
                format!("{WSL_RESTORE}; {BASH_OSC7}"),
            ));
            let mut forwarded = vec!["PROMPT_COMMAND/u"];
            if let Some(dir) = input.restore_cwd {
                plan.env.push(("EMBER_CWD".into(), dir.into()));
                forwarded.push("EMBER_CWD/u");
            }
            plan.env.push((
                "WSLENV".into(),
                merge_wslenv(input.inherited_wslenv, &forwarded),
            ));
        }
        ShellKind::Pwsh => {
            if !has_any_arg(
                &plan.args,
                &[
                    "-command",
                    "-c",
                    "-file",
                    "-f",
                    "-encodedcommand",
                    "-e",
                    "-ec",
                ],
            ) {
                if !has_any_arg(&plan.args, &["-noexit"]) {
                    plan.args.push("-NoExit".into());
                }
                plan.args.push("-EncodedCommand".into());
                plan.args.push(encode_pwsh(PWSH_HOOK));
            }
            apply_restore_dir(
                &mut plan,
                input.restore_cwd.map(str::to_string),
                input.dir_exists,
            );
        }
        ShellKind::Other => {}
    }
    plan
}

fn apply_restore_dir(
    plan: &mut SpawnPlan,
    dir: Option<String>,
    dir_exists: &dyn Fn(&Path) -> bool,
) {
    if let Some(dir) = dir.map(PathBuf::from) {
        if dir_exists(&dir) {
            plan.cwd = dir;
        } else {
            tracing::warn!(dir = %dir.display(), "saved folder no longer exists; using profile folder");
        }
    }
}

fn has_any_arg(args: &[String], names: &[&str]) -> bool {
    args.iter()
        .any(|a| names.iter().any(|n| a.eq_ignore_ascii_case(n)))
}

fn encode_pwsh(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// `/d/code` → `D:/code`. Paths outside a drive mount (e.g. `/tmp`) can't be restored.
pub fn msys_to_windows(path: &str) -> Option<String> {
    let rest = path.strip_prefix('/')?;
    let mut chars = rest.chars();
    let drive = chars.next().filter(char::is_ascii_alphabetic)?;
    let tail = chars.as_str();
    if !(tail.is_empty() || tail.starts_with('/')) {
        return None;
    }
    let tail = tail.strip_prefix('/').unwrap_or("");
    Some(format!("{}:/{}", drive.to_ascii_uppercase(), tail))
}

/// Appends entries to WSLENV without dropping or duplicating existing ones.
pub fn merge_wslenv(existing: Option<&str>, add: &[&str]) -> String {
    let mut entries: Vec<String> = existing
        .unwrap_or("")
        .split(':')
        .filter(|e| !e.is_empty())
        .map(str::to_string)
        .collect();
    for entry in add {
        let name = entry.split('/').next().unwrap_or(entry);
        if !entries.iter().any(|e| e.split('/').next() == Some(name)) {
            entries.push((*entry).to_string());
        }
    }
    entries.join(":")
}

#[cfg(test)]
#[path = "shell_hooks_test.rs"]
mod tests;
