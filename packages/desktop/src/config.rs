use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::fsutil::{load_user_file, write_atomic, StartupWarning};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme: String,
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub window_padding: u32,
    pub acrylic: bool,
    pub opacity: f32,
    /// Cursor shape: "block" | "bar" | "underline"
    pub cursor_style: String,
    pub cursor_blink: bool,
    /// Draw bold ANSI text using the bright color variant
    pub bold_in_bright: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub default_profile: String,
    pub confirm_close_running: bool,
    pub scrollback_lines: usize,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "ember-dark".into(),
            font_family: "JetBrains Mono, Cascadia Code, Fira Code, monospace".into(),
            font_size: 13,
            line_height: 1.2,
            window_padding: 8,
            acrylic: true,
            opacity: 0.95,
            cursor_style: "block".into(),
            cursor_blink: true,
            bold_in_bright: true,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            default_profile: "default".into(),
            confirm_close_running: true,
            scrollback_lines: 10000,
        }
    }
}

impl Config {
    pub fn load(quarantine_broken: bool) -> (Self, Option<StartupWarning>) {
        match config_path() {
            Some(path) => Self::load_from(&path, quarantine_broken),
            None => (Self::default(), None),
        }
    }

    pub fn load_from(path: &Path, quarantine_broken: bool) -> (Self, Option<StartupWarning>) {
        load_user_file(path, quarantine_broken, |text| {
            toml::from_str(text).map_err(|e| e.to_string())
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path().ok_or_else(|| anyhow::anyhow!("no config directory"))?;
        self.save_to(&path)
    }

    /// Refuses to overwrite a config.toml the user is part-way through editing.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Err(e) = toml::from_str::<Config>(&text) {
                anyhow::bail!("config.toml has an error; fix it before changing settings: {e}");
            }
        }
        write_atomic(path, toml::to_string_pretty(self)?.as_bytes())?;
        Ok(())
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ember").join("config.toml"))
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
