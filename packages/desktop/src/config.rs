use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    pub fn load() -> Self {
        match config_path() {
            Some(path) if path.exists() => match std::fs::read_to_string(&path) {
                Ok(text) => toml::from_str(&text).unwrap_or_default(),
                Err(_) => Config::default(),
            },
            _ => Config::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let text = toml::to_string_pretty(self)?;
            std::fs::write(path, text)?;
        }
        Ok(())
    }
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ember").join("config.toml"))
}
