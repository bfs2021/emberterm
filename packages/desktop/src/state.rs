use std::collections::HashMap;
use std::sync::Mutex;

use crate::config::Config;
use crate::profile::Profile;
use crate::terminal::session::{Session, SessionId};

pub struct AppState {
    pub sessions: Mutex<HashMap<SessionId, Session>>,
    pub config: Mutex<Config>,
    pub profiles: Mutex<Vec<Profile>>,
    /// Keeps the file watcher alive for the duration of the app.
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
    next_id: Mutex<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            config: Mutex::new(Config::load()),
            profiles: Mutex::new(crate::profile::load_profiles()),
            watcher: Mutex::new(None),
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
