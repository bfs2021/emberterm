use std::path::PathBuf;
use std::sync::OnceLock;

/// Looks for `name` in a PATH-style list without spawning `where`/`which`.
pub fn find_in_path(name: &str, path_var: &str) -> Option<PathBuf> {
    std::env::split_paths(path_var)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

pub fn on_path(name: &str) -> bool {
    std::env::var("PATH")
        .map(|p| find_in_path(name, &p).is_some())
        .unwrap_or(false)
}

/// Platform default shell, resolved once per process.
pub fn default_shell() -> String {
    static SHELL: OnceLock<String> = OnceLock::new();
    SHELL.get_or_init(detect_default_shell).clone()
}

fn detect_default_shell() -> String {
    #[cfg(windows)]
    {
        for candidate in ["pwsh.exe", "powershell.exe"] {
            if on_path(candidate) {
                return candidate.to_string();
            }
        }
        "cmd.exe".to_string()
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

pub fn default_args() -> Vec<String> {
    if cfg!(windows) {
        vec!["-NoLogo".into()]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "which_test.rs"]
mod tests;
