use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{App, Emitter, Manager};

use crate::state::lock_or_recover;

pub fn setup(app: &mut App) -> anyhow::Result<()> {
    let app_handle = app.handle().clone();

    let mut watcher: RecommendedWatcher =
        recommended_watcher(move |res: notify::Result<Event>| {
            let Ok(event) = res else { return };

            let relevant = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) | EventKind::Any
            );
            if !relevant {
                return;
            }

            let state = app_handle.state::<crate::state::AppState>();

            for path in &event.paths {
                if path.file_name().is_some_and(|n| n == "config.toml") {
                    let new_cfg = crate::config::Config::load();
                    *lock_or_recover(&state.config) = new_cfg.clone();
                    let _ = app_handle.emit("config:updated", new_cfg);
                } else if path.extension().is_some_and(|e| e == "toml") {
                    let new_profiles = crate::profile::load_profiles();
                    *lock_or_recover(&state.profiles) = new_profiles.clone();
                    let _ = app_handle.emit("profiles:updated", new_profiles);
                }
            }
        })?;

    // Watch config directory (non-recursive; catches config.toml create/modify)
    if let Some(cfg_path) = crate::config::config_path() {
        if let Some(dir) = cfg_path.parent() {
            let _ = std::fs::create_dir_all(dir);
            let _ = watcher.watch(dir, RecursiveMode::NonRecursive);
        }
    }

    // Watch profiles directory
    if let Some(profiles_dir) = crate::profile::profiles_dir() {
        let _ = std::fs::create_dir_all(&profiles_dir);
        let _ = watcher.watch(&profiles_dir, RecursiveMode::NonRecursive);
    }

    let state = app.state::<crate::state::AppState>();
    *lock_or_recover(&state.watcher) = Some(watcher);

    Ok(())
}
