use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::fsutil::{load_user_file, write_atomic, StartupWarning};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub env: std::collections::HashMap<String, String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: "default".into(),
            name: "Default Shell".into(),
            shell: crate::terminal::which::default_shell(),
            args: crate::terminal::which::default_args(),
            cwd: "~".into(),
            env: std::collections::HashMap::new(),
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

pub fn load_profiles(quarantine_broken: bool) -> (Vec<Profile>, Vec<StartupWarning>) {
    match profiles_dir() {
        Some(dir) => load_profiles_from(&dir, quarantine_broken),
        None => (vec![Profile::default()], Vec::new()),
    }
}

pub fn load_profiles_from(
    dir: &Path,
    quarantine_broken: bool,
) -> (Vec<Profile>, Vec<StartupWarning>) {
    let mut profiles = Vec::new();
    let mut warnings = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for path in entries.flatten().map(|e| e.path()) {
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let (profile, warning) = load_user_file(&path, quarantine_broken, |text| {
                toml::from_str::<Profile>(text)
                    .map(Some)
                    .map_err(|e| e.to_string())
            });
            profiles.extend(profile);
            warnings.extend(warning);
        }
    }
    if profiles.is_empty() {
        profiles.push(Profile::default());
    }
    (profiles, warnings)
}

pub fn save_profile(profile: &Profile) -> Result<()> {
    let dir = profiles_dir().ok_or_else(|| anyhow::anyhow!("no config dir"))?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.toml", profile.id));
    let text = toml::to_string_pretty(profile)?;
    write_atomic(&path, text.as_bytes())?;
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

#[cfg(test)]
#[path = "profile_test.rs"]
mod tests;
