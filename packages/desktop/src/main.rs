#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod config;
mod ipc;
mod platform;
mod profile;
mod quake;
mod state;
mod terminal;
mod watcher;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("emberterm=debug")),
        )
        .init();

    // Build the global shortcut (Ctrl+`) that triggers quake mode.
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};
    let quake_shortcut = tauri_plugin_global_shortcut::Shortcut::new(
        Some(Modifiers::CONTROL),
        Code::Backquote,
    );
    let qs = quake_shortcut.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(
                    move |app, sc, event: tauri_plugin_global_shortcut::ShortcutEvent| {
                        if sc == &qs && event.state() == ShortcutState::Pressed {
                            quake::toggle(app);
                        }
                    },
                )
                .build(),
        )
        .manage(state::AppState::new())
        .setup(move |app| {
            platform::setup_window(app)?;
            setup_tray(app)?;
            // Register quake hotkey.
            app.global_shortcut().register(quake_shortcut.clone())
                .unwrap_or_else(|e| tracing::warn!("global shortcut registration failed: {e}"));
            if let Err(e) = watcher::setup(app) {
                tracing::warn!("file watcher setup failed: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::session_create,
            ipc::session_write,
            ipc::session_resize,
            ipc::session_close,
            ipc::config_get,
            ipc::config_update,
            ipc::profile_list,
            ipc::profile_upsert,
            ipc::profile_delete,
            ipc::block_get,
            ipc::block_search,
            ipc::agent_respond,
            ipc::git_status,
            ipc::query_agent_version,
            ipc::config_is_first_run,
            ipc::quake_toggle,
            ipc::detect_builtin_profiles,
            ipc::set_window_opacity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EmberTerm");
}

/// Build the system tray icon. Left-click toggles quake mode.
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let icon = app.default_window_icon()
        .cloned()
        .unwrap_or_else(|| tauri::image::Image::new(&[], 1, 1));

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("EmberTerm — Ctrl+` to toggle")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                quake::toggle(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
