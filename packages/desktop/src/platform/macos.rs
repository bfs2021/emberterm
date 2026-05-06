use tauri::App;
use tauri::Manager;
use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};

pub fn apply_effect(app: &mut App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").unwrap();
    if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None) {
        tracing::debug!("vibrancy unavailable: {e}");
    }
    Ok(())
}
