use std::collections::HashMap;
use std::sync::Mutex;

use crate::terminal::session::{Session, SessionId};

pub struct AppState {
    pub sessions: Mutex<HashMap<SessionId, Session>>,
    next_id: Mutex<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn next_session_id(&self) -> SessionId {
        let mut n = self.next_id.lock().unwrap();
        let id = format!("session-{}", *n);
        *n += 1;
        id
    }
}
