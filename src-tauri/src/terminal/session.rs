use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::agent::tools::ActiveTools;
use super::blocks::Block;

pub type SessionId = String;

pub struct Session {
    pub id: SessionId,
    /// PTY master — used for resize.
    pub master: Box<dyn portable_pty::MasterPty + Send>,
    /// Write half of the PTY master — used for sending keystrokes.
    pub writer: Mutex<Box<dyn std::io::Write + Send>>,
    pub title: String,
    pub cwd: Option<PathBuf>,
    /// Active tool calls from any agent running in this session.
    pub active_tools: Mutex<ActiveTools>,
    /// Name/version of the connected agent, if any.
    pub agent_name: Mutex<Option<String>>,
    /// All completed and in-progress blocks for this session (OSC 133 driven).
    pub blocks: Arc<Mutex<Vec<Block>>>,
}
