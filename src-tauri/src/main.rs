#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod config;
mod ipc;
mod platform;
mod profile;
mod state;
mod terminal;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("emberterm=debug")),
        )
        .init();

    tauri::Builder::default()
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            ipc::session_create,
            ipc::session_write,
            ipc::session_resize,
            ipc::session_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EmberTerm");
}
