use tauri::App;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

/// Apply platform-specific window effects (Mica/vibrancy/noop).
pub fn setup_window(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    windows::apply_effect(app)?;

    #[cfg(target_os = "macos")]
    macos::apply_effect(app)?;

    #[cfg(target_os = "linux")]
    linux::apply_effect(app)?;

    Ok(())
}
