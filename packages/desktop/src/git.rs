use std::path::Path;
use std::process::Command;

/// The folder comes from OSC 7, which any program output can forge. A network path would make
/// Windows start SMB auth (leaking the user's NTLM hash), so only an existing local folder is used.
pub fn safe_cwd(cwd: &str) -> Option<&str> {
    let network = cwd.starts_with("\\\\") || cwd.starts_with("//");
    (!network && Path::new(cwd).is_dir()).then_some(cwd)
}

/// A `git` command for the status bar. The repository may be untrusted, so config that runs
/// programs (`core.fsmonitor`) is off, and git must not write to the repository or prompt.
/// The release build has no console, so Windows would flash a new console window per call
/// unless it is told not to.
pub fn command(cwd: &str) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["-c", "core.fsmonitor=false", "--no-optional-locks"])
        .current_dir(cwd)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(test)]
#[path = "git_test.rs"]
mod tests;
