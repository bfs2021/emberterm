use std::sync::{Arc, Mutex};

pub type SessionId = String;

pub struct Session {
    /// PTY master — used for resize.
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    /// Write half of the PTY master, used by `session_write`.
    pub writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    /// Process id of the shell, for "is something still running in it" checks.
    pub pid: Option<u32>,
}

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
