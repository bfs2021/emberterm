use std::path::{Path, PathBuf};

use super::*;

fn input<'a>(
    kind: ShellKind,
    args: &'a [String],
    restore: Option<&'a str>,
    exists: &'a dyn Fn(&Path) -> bool,
) -> HookInput<'a> {
    HookInput {
        kind,
        profile_args: args,
        profile_cwd: PathBuf::from("C:/Users/me"),
        restore_cwd: restore,
        inherited_prompt_command: false,
        inherited_wslenv: None,
        dir_exists: exists,
    }
}

fn env<'a>(plan: &'a SpawnPlan, key: &str) -> Option<&'a str> {
    plan.env
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

#[test]
fn detects_kinds() {
    assert_eq!(
        shell_kind(r"C:\Program Files\Git\bin\bash.exe"),
        ShellKind::Bash
    );
    assert_eq!(shell_kind("wsl.exe"), ShellKind::Wsl);
    assert_eq!(shell_kind("PWSH.EXE"), ShellKind::Pwsh);
    assert_eq!(shell_kind("powershell.exe"), ShellKind::Pwsh);
    assert_eq!(shell_kind("cmd.exe"), ShellKind::Other);
}

#[test]
fn msys_paths_convert_to_windows() {
    assert_eq!(
        msys_to_windows("/d/code/api").as_deref(),
        Some("D:/code/api")
    );
    assert_eq!(msys_to_windows("/c").as_deref(), Some("C:/"));
    assert_eq!(msys_to_windows("/tmp/x"), None);
    assert_eq!(msys_to_windows("D:/x"), None);
}

#[test]
fn bash_gets_osc7_hook_and_restored_dir() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Bash, &[], Some("/d/code/api"), &yes));
    assert!(env(&plan, "PROMPT_COMMAND")
        .unwrap()
        .contains("]7;file://localhost%s"));
    if cfg!(windows) {
        assert_eq!(plan.cwd, PathBuf::from("D:/code/api"));
    }
}

#[test]
fn bash_respects_inherited_prompt_command() {
    let yes = |_: &Path| true;
    let mut i = input(ShellKind::Bash, &[], None, &yes);
    i.inherited_prompt_command = true;
    assert!(env(&prepare(&i), "PROMPT_COMMAND").is_none());
}

#[test]
fn missing_restore_dir_falls_back_to_profile_cwd() {
    let no = |_: &Path| false;
    let plan = prepare(&input(ShellKind::Pwsh, &[], Some("D:/gone"), &no));
    assert_eq!(plan.cwd, PathBuf::from("C:/Users/me"));
}

#[test]
fn wsl_forwards_hook_and_restore_dir_via_wslenv() {
    let yes = |_: &Path| true;
    let mut i = input(ShellKind::Wsl, &[], Some("/home/me/api"), &yes);
    i.inherited_wslenv = Some("USERPROFILE/p");
    let plan = prepare(&i);
    assert_eq!(env(&plan, "EMBER_CWD"), Some("/home/me/api"));
    let pc = env(&plan, "PROMPT_COMMAND").unwrap();
    assert!(pc.starts_with("[ -n \"$EMBER_CWD\" ]"));
    assert!(pc.contains("]7;file://localhost%s"));
    assert_eq!(
        env(&plan, "WSLENV"),
        Some("USERPROFILE/p:PROMPT_COMMAND/u:EMBER_CWD/u")
    );
    assert_eq!(plan.cwd, PathBuf::from("C:/Users/me"));
}

#[test]
fn wsl_without_restore_sets_no_ember_cwd() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Wsl, &[], None, &yes));
    assert!(env(&plan, "EMBER_CWD").is_none());
}

#[test]
fn merge_wslenv_does_not_duplicate() {
    assert_eq!(
        merge_wslenv(
            Some("PROMPT_COMMAND/u"),
            &["PROMPT_COMMAND/u", "EMBER_CWD/u"]
        ),
        "PROMPT_COMMAND/u:EMBER_CWD/u"
    );
    assert_eq!(merge_wslenv(None, &["A/u"]), "A/u");
    assert_eq!(merge_wslenv(Some(""), &["A/u"]), "A/u");
}

#[test]
fn pwsh_gets_encoded_hook() {
    let yes = |_: &Path| true;
    let args = vec!["-NoLogo".to_string()];
    let plan = prepare(&input(ShellKind::Pwsh, &args, Some("D:/code"), &yes));
    assert_eq!(&plan.args[..3], &["-NoLogo", "-NoExit", "-EncodedCommand"]);
    let decoded = decode_utf16le_base64(&plan.args[3]);
    assert!(decoded.contains("]7;file://localhost/"));
    assert!(decoded.contains("$global:__ember_prompt"));
    assert_eq!(plan.cwd, PathBuf::from("D:/code"));
}

#[test]
fn pwsh_hook_skipped_when_profile_has_command() {
    let yes = |_: &Path| true;
    let args = vec!["-Command".to_string(), "Get-Date".to_string()];
    let plan = prepare(&input(ShellKind::Pwsh, &args, None, &yes));
    assert_eq!(plan.args, args);
}

#[test]
fn pwsh_does_not_duplicate_noexit() {
    let yes = |_: &Path| true;
    let args = vec!["-NoExit".to_string()];
    let plan = prepare(&input(ShellKind::Pwsh, &args, None, &yes));
    assert_eq!(
        plan.args
            .iter()
            .filter(|a| a.eq_ignore_ascii_case("-noexit"))
            .count(),
        1
    );
}

#[test]
fn other_shells_get_nothing() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Other, &[], Some("D:/x"), &yes));
    assert!(plan.env.is_empty());
    assert_eq!(plan.cwd, PathBuf::from("C:/Users/me"));
}

#[test]
fn bash_hook_escapes_percent_and_control_chars_and_keeps_status() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Bash, &[], None, &yes));
    let pc = env(&plan, "PROMPT_COMMAND").unwrap();
    // `%` must be encoded so a folder literally named "a%20b" round-trips,
    // ESC/BEL must not be able to end the OSC early, and `$?` must survive
    // for whatever the user appends after the hook.
    assert!(pc.contains("${PWD//%/%25}"), "{pc}");
    assert!(pc.contains("%1B") && pc.contains("%07"), "{pc}");
    assert!(
        pc.contains("local s=$?") && pc.contains("return $s"),
        "{pc}"
    );
}

#[test]
fn pwsh_hook_percent_encodes_the_path_as_ascii() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Pwsh, &[], None, &yes));
    let decoded = decode_utf16le_base64(&plan.args[2]);
    assert!(decoded.contains("EscapeDataString"), "{decoded}");
}

#[test]
fn pwsh_hook_runs_the_original_prompt_before_touching_status() {
    let yes = |_: &Path| true;
    let plan = prepare(&input(ShellKind::Pwsh, &[], None, &yes));
    let decoded = decode_utf16le_base64(&plan.args[2]);
    let original = decoded
        .find("$o = & $global:__ember_prompt")
        .expect("calls original first");
    let write = decoded.find("[Console]::Write").unwrap();
    assert!(original < write, "{decoded}");
}

fn decode_utf16le_base64(s: &str) -> String {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(s).unwrap();
    let units: Vec<u16> = bytes
        .chunks(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).unwrap()
}
