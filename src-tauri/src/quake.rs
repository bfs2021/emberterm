//! Quake-mode: a drop-down terminal toggled by a global hotkey or tray click.
//!
//! When activated:
//!   • If the window is visible → hide it.
//!   • If the window is hidden  → reposition to the top of the screen at full
//!     width × 40% height, then show and focus it.
//!
//! The normal window geometry is not preserved — after dismissing quake mode
//! the window is restored with `window.unminimize()` / `window.show()` as
//! usual.  Geometry save/restore is deferred to v1.0.

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

const QUAKE_HEIGHT_RATIO: f64 = 0.40;

/// Toggle quake mode on the main window.
pub fn toggle(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        tracing::warn!("quake toggle: main window not found");
        return;
    };

    let visible = window.is_visible().unwrap_or(false);

    if visible {
        if let Err(e) = window.hide() {
            tracing::error!(error = %e, "quake: failed to hide window");
        }
    } else {
        // Position the window at the top of the primary monitor.
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let ms = monitor.size();
            let quake_h = (ms.height as f64 * QUAKE_HEIGHT_RATIO).round() as u32;
            let _ = window.set_size(PhysicalSize::new(ms.width, quake_h));
            let _ = window.set_position(PhysicalPosition::new(0i32, 0));
        }
        if let Err(e) = window.show() {
            tracing::error!(error = %e, "quake: failed to show window");
            return;
        }
        let _ = window.set_focus();
    }
}
