use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::config::Config;
use crate::fsutil::StartupWarning;
use crate::profile::Profile;
use crate::terminal::session::{Session, SessionId};

pub struct AppState {
    pub sessions: Mutex<HashMap<SessionId, Session>>,
    pub config: Mutex<Config>,
    pub profiles: Mutex<Vec<Profile>>,
    /// Keeps the file watcher alive for the duration of the app.
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
    /// Warnings collected while loading config and profiles at startup.
    pub warnings: Mutex<Vec<StartupWarning>>,
    next_id: Mutex<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (config, config_warning) = Config::load(true);
        let (profiles, profile_warnings) = crate::profile::load_profiles(true);
        Self {
            sessions: Mutex::new(HashMap::new()),
            config: Mutex::new(config),
            profiles: Mutex::new(profiles),
            watcher: Mutex::new(None),
            warnings: Mutex::new(config_warning.into_iter().chain(profile_warnings).collect()),
            next_id: Mutex::new(1),
        }
    }

    pub fn next_session_id(&self) -> SessionId {
        let mut n = lock_or_recover(&self.next_id);
        let id = format!("session-{}", *n);
        *n += 1;
        id
    }
}

/// Locks `m`, recovering the data if another thread panicked while holding it.
/// A poisoned lock must not take down every tab.
pub fn lock_or_recover<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
