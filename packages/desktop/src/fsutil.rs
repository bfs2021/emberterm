use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Writes `bytes` so readers (and a crash mid-write) never observe a partial file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = with_suffix(path, ".tmp");
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Moves a file that failed to parse aside so a later save cannot overwrite
/// the user's text. Returns the backup path.
pub fn quarantine(path: &Path) -> std::io::Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = with_suffix(path, &format!(".broken-{ts}"));
    std::fs::rename(path, &backup)?;
    Ok(backup)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// A user-editable file that could not be read or parsed.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StartupWarning {
    pub file: String,
    pub message: String,
    pub backup: Option<String>,
}

impl StartupWarning {
    fn new(path: &Path, message: String, backup: Option<PathBuf>) -> Self {
        Self {
            file: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            message,
            backup: backup.map(|b| b.display().to_string()),
        }
    }
}

/// Reads and parses a user-editable file. A parse failure yields the default
/// value plus a warning; at startup (`quarantine_broken`) the file is also moved
/// aside so the next save cannot overwrite it. Missing files are not an error.
pub fn load_user_file<T: Default>(
    path: &Path,
    quarantine_broken: bool,
    parse: impl FnOnce(&str) -> Result<T, String>,
) -> (T, Option<StartupWarning>) {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (T::default(), None),
        Err(e) => {
            return (
                T::default(),
                Some(StartupWarning::new(path, e.to_string(), None)),
            )
        }
    };
    match parse(&text) {
        Ok(value) => (value, None),
        Err(message) => {
            tracing::warn!(file = %path.display(), %message, "user file failed to parse");
            let backup = if quarantine_broken {
                quarantine(path).ok()
            } else {
                None
            };
            (
                T::default(),
                Some(StartupWarning::new(path, message, backup)),
            )
        }
    }
}

#[cfg(test)]
#[path = "fsutil_test.rs"]
mod tests;
