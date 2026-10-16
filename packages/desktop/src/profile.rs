use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub shell: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub env: std::collections::HashMap<String, String>,
    pub theme: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<u32>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: "default".into(),
            name: "Default Shell".into(),
            icon: "terminal".into(),
            shell: default_shell(),
            args: default_args(),
            cwd: "~".into(),
            env: std::collections::HashMap::new(),
            theme: None,
            font_family: None,
            font_size: None,
        }
    }
}

impl Profile {
    pub fn resolved_cwd(&self) -> PathBuf {
        if self.cwd == "~" {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        } else if self.cwd.starts_with("~/") || self.cwd.starts_with("~\\") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(&self.cwd[2..])
        } else {
            PathBuf::from(&self.cwd)
        }
    }
}

pub fn profiles_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ember").join("profiles"))
}

pub fn load_profiles() -> Vec<Profile> {
    let dir = match profiles_dir() {
        Some(d) => d,
        None => return vec![Profile::default()],
    };

    if !dir.exists() {
        return vec![Profile::default()];
    }

    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Some(p) = load_profile_file(&path) {
                    profiles.push(p);
                }
            }
        }
    }

    if profiles.is_empty() {
        profiles.push(Profile::default());
    }

    profiles
}

fn load_profile_file(path: &Path) -> Option<Profile> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

pub fn save_profile(profile: &Profile) -> Result<()> {
    let dir = profiles_dir().ok_or_else(|| anyhow::anyhow!("no config dir"))?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.toml", profile.id));
    let text = toml::to_string_pretty(profile)?;
    std::fs::write(path, text)?;
    Ok(())
}

pub fn delete_profile(id: &str) -> Result<()> {
    let dir = profiles_dir().ok_or_else(|| anyhow::anyhow!("no config dir"))?;
    let path = dir.join(format!("{}.toml", id));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn default_shell() -> String {
    #[cfg(windows)]
    {
        if std::process::Command::new("where")
            .arg("pwsh.exe")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return "pwsh.exe".into();
        }
        "cmd.exe".into()
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
    }
}

fn default_args() -> Vec<String> {
    #[cfg(windows)]
    {
        vec!["-NoLogo".into()]
    }
    #[cfg(not(windows))]
    {
        vec![]
    }
}
