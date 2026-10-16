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
