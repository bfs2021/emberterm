use super::*;

fn p(pid: u32, parent: Option<u32>, name: &str) -> ProcInfo {
    ProcInfo {
        pid,
        parent,
        name: name.into(),
    }
}

#[test]
fn an_idle_shell_is_not_busy() {
    let procs = [p(10, Some(1), "bash.exe"), p(99, Some(1), "explorer.exe")];
    assert!(!has_busy_child(10, &procs));
}

#[test]
fn a_running_child_makes_the_shell_busy() {
    let procs = [p(10, Some(1), "bash.exe"), p(11, Some(10), "sleep.exe")];
    assert!(has_busy_child(10, &procs));
}

#[test]
fn a_grandchild_counts() {
    let procs = [
        p(10, Some(1), "pwsh.exe"),
        p(11, Some(10), "node.exe"),
        p(12, Some(11), "esbuild.exe"),
    ];
    assert!(has_busy_child(10, &procs));
}

#[test]
fn console_helpers_are_ignored() {
    let procs = [
        p(10, Some(1), "wsl.exe"),
        p(11, Some(10), "conhost.exe"),
        p(12, Some(10), "OpenConsole.exe"),
        p(13, Some(10), "wslhost.exe"),
    ];
    assert!(!has_busy_child(10, &procs));
}

#[test]
fn unrelated_processes_and_loops_do_not_count() {
    let procs = [
        p(10, Some(1), "bash.exe"),
        p(20, Some(21), "a.exe"),
        p(21, Some(20), "b.exe"), // parent cycle, never reaches 10
    ];
    assert!(!has_busy_child(10, &procs));
}

#[test]
fn a_launcher_that_starts_the_real_shell_is_not_a_running_program() {
    // Git for Windows: bin/bash.exe starts usr/bin/bash.exe.
    let procs = [p(10, Some(1), "bash.exe"), p(11, Some(10), "bash.exe")];
    assert!(!has_busy_child(10, &procs));
}

#[test]
fn a_program_under_the_real_shell_still_counts() {
    let procs = [
        p(10, Some(1), "bash.exe"),
        p(11, Some(10), "bash.exe"),
        p(12, Some(11), "sleep.exe"),
    ];
    assert!(has_busy_child(10, &procs));
}

#[test]
fn shell_name_matching_ignores_case() {
    let procs = [p(10, Some(1), "Bash.exe"), p(11, Some(10), "bash.EXE")];
    assert!(!has_busy_child(10, &procs));
}
