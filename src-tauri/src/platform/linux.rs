use tauri::App;

/// Linux window setup. Window blur/vibrancy is not available via the
/// window-vibrancy crate on Linux; the webview uses a solid background
/// as defined by the `--bg` CSS token. Future: investigate GTK/KWin
/// compositor hints for optional blur on Wayland compositors that support it.
pub fn apply_effect(_app: &mut App) -> tauri::Result<()> {
    Ok(())
}
