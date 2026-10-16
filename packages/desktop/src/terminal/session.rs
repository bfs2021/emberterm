use std::sync::{Arc, Mutex};

use super::flow::FlowControl;

pub type SessionId = String;

pub struct Session {
    /// PTY master — used for resize.
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    /// Write half of the PTY master, used by `session_write`.
    pub writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    /// Process id of the shell, for "is something still running in it" checks.
    pub pid: Option<u32>,
    /// Output sent to the webview and not yet acked; `session_ack` reports progress.
    pub(crate) flow: Arc<FlowControl>,
}

impl Drop for Session {
    /// A sender blocked on acks would otherwise wait forever for a closed tab.
    fn drop(&mut self) {
        self.flow.close();
    }
}

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
