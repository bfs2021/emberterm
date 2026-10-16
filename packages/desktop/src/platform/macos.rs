use tauri::App;
use tauri::Manager;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

pub fn apply_effect(app: &mut App) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        tracing::warn!("main window not found; skipping window effect");
        return Ok(());
    };
    if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None) {
        tracing::debug!("vibrancy unavailable: {e}");
    }
    Ok(())
}
