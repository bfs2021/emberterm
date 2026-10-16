use super::*;

#[test]
fn loads_valid_config_and_ignores_unknown_sections() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "[appearance]\nfont_size = 15\n[agent]\nosc_7777_enabled = true\n",
    )
    .unwrap();
    let (cfg, w) = Config::load_from(&path, true);
    assert!(w.is_none());
    assert_eq!(cfg.appearance.font_size, 15);
}

#[test]
fn broken_config_at_startup_is_quarantined() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[appearance\nfont_size = ").unwrap();
    let (cfg, w) = Config::load_from(&path, true);
    assert_eq!(
        cfg.appearance.font_size,
        Config::default().appearance.font_size
    );
    let w = w.unwrap();
    assert!(w.message.contains("line"), "{}", w.message);
    assert!(w.backup.is_some());
    assert!(!path.exists());
}

#[test]
fn save_refuses_to_overwrite_broken_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[appearance\n").unwrap();
    assert!(Config::default().save_to(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "[appearance\n");
}

#[test]
fn save_writes_when_file_valid_or_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    Config::default().save_to(&path).unwrap();
    Config::default().save_to(&path).unwrap();
    let (_, w) = Config::load_from(&path, true);
    assert!(w.is_none());
}

#[test]
fn first_run_means_no_config_and_no_quarantined_config() {
    let dir = tempfile::tempdir().unwrap();
    assert!(is_first_run_in(dir.path()));
    std::fs::write(dir.path().join("config.toml.broken-123"), b"[[[").unwrap();
    assert!(
        !is_first_run_in(dir.path()),
        "a quarantined config is not a first run"
    );
}

#[test]
fn existing_config_is_not_first_run() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), b"").unwrap();
    assert!(!is_first_run_in(dir.path()));
}

#[test]
fn save_refuses_to_overwrite_non_utf8_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    // UTF-16LE with BOM, as Windows PowerShell 5.1 `>` writes it.
    let bytes: Vec<u8> = vec![0xFF, 0xFE, b'[', 0, b'a', 0, b']', 0];
    std::fs::write(&path, &bytes).unwrap();
    assert!(Config::default().save_to(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn config_with_removed_window_effect_keys_still_loads() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "[appearance]
font_size = 16
acrylic = true
opacity = 0.75
",
    )
    .unwrap();
    let (cfg, w) = Config::load_from(&path, true);
    assert!(w.is_none());
    assert_eq!(cfg.appearance.font_size, 16);
}
