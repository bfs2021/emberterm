use tauri::App;
use tauri::Manager;
use window_vibrancy::apply_mica;

pub fn apply_effect(app: &mut App) -> tauri::Result<()> {
    let window = app.get_webview_window("main").unwrap();
    // Mica is available on Windows 11. apply_mica returns an error on older
    // Windows versions; we silently ignore it and fall back to a solid bg.
    if let Err(e) = apply_mica(&window, Some(true)) {
        tracing::debug!("Mica unavailable, using solid background: {e}");
    }
    Ok(())
}
