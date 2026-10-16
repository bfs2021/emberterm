use std::collections::BTreeMap;
use std::sync::OnceLock;

/// One installed font family, as offered in the font picker.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FontFamily {
    pub name: String,
    pub monospaced: bool,
}

/// Collapses (family name, monospaced) pairs from every font face into one entry per
/// family, sorted case-insensitively. Hidden (`.x`) and vertical-writing (`@x`) names
/// are skipped; a family counts as monospaced if any of its faces is.
pub fn group_families(faces: impl IntoIterator<Item = (String, bool)>) -> Vec<FontFamily> {
    let mut by_name: BTreeMap<String, FontFamily> = BTreeMap::new();
    for (name, monospaced) in faces {
        let name = name.trim().to_string();
        if name.is_empty() || name.starts_with('@') || name.starts_with('.') {
            continue;
        }
        by_name
            .entry(name.to_lowercase())
            .and_modify(|f| f.monospaced |= monospaced)
            .or_insert(FontFamily { name, monospaced });
    }
    by_name.into_values().collect()
}

/// Scanning the font directories takes a noticeable moment, so do it once per run.
fn installed_families() -> &'static [FontFamily] {
    static CACHE: OnceLock<Vec<FontFamily>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        group_families(db.faces().flat_map(|face| {
            let monospaced = face.monospaced;
            face.families
                .iter()
                .map(move |(name, _)| (name.clone(), monospaced))
        }))
    })
}

/// Font families installed on this computer (for the Appearance font picker).
#[tauri::command]
pub async fn font_list() -> Vec<FontFamily> {
    tokio::task::spawn_blocking(|| installed_families().to_vec())
        .await
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "fonts_test.rs"]
mod tests;
