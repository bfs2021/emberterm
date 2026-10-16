/// One running process, reduced to what the busy check needs.
#[derive(Debug, Clone)]
pub struct ProcInfo {
    pub pid: u32,
    pub parent: Option<u32>,
    pub name: String,
}

/// Console plumbing that hangs off a shell without being something the user is running.
const HELPERS: [&str; 3] = ["conhost.exe", "openconsole.exe", "wslhost.exe"];

fn is_helper(name: &str) -> bool {
    let lower = name.to_lowercase();
    HELPERS.iter().any(|h| *h == lower)
}

/// True when the shell `root` has a live descendant that is a real program, i.e. something the
/// user would lose by closing the tab. Console plumbing is ignored, and so is a child that is
/// the same executable as the shell: Git for Windows' bin/bash.exe is a launcher that starts
/// the real bash.exe, so that child is the shell itself and its children are what count.
pub fn has_busy_child(root: u32, procs: &[ProcInfo]) -> bool {
    let Some(root_name) = procs
        .iter()
        .find(|p| p.pid == root)
        .map(|p| p.name.to_lowercase())
    else {
        return false;
    };
    let mut seen = vec![root];
    let mut frontier = vec![root];
    while let Some(parent) = frontier.pop() {
        for child in procs.iter().filter(|p| p.parent == Some(parent)) {
            if seen.contains(&child.pid) || is_helper(&child.name) {
                continue;
            }
            seen.push(child.pid);
            if child.name.to_lowercase() == root_name {
                frontier.push(child.pid); // the shell again: look through it
            } else {
                return true;
            }
        }
    }
    false
}

/// Snapshot of every running process.
pub fn snapshot() -> Vec<ProcInfo> {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.processes()
        .iter()
        .map(|(pid, p)| ProcInfo {
            pid: pid.as_u32(),
            parent: p.parent().map(|pp| pp.as_u32()),
            name: p.name().to_string_lossy().into_owned(),
        })
        .collect()
}

#[cfg(test)]
#[path = "busy_test.rs"]
mod tests;
