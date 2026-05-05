use std::path::PathBuf;
use std::sync::Mutex;

pub type SessionId = String;

pub struct Session {
    pub id: SessionId,
    /// PTY master — used for resize.
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    /// Write half of the PTY master — used for sending keystrokes.
    pub writer: Mutex<Box<dyn std::io::Write + Send>>,
    pub title: String,
    pub cwd: Option<PathBuf>,
}
