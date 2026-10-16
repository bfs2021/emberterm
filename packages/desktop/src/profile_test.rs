use super::*;

#[test]
fn broken_profile_is_skipped_with_warning() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("good.toml"),
        "id = \"good\"\nname = \"Good\"\nshell = \"bash.exe\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("bad.toml"), "id = ").unwrap();
    let (profiles, warnings) = load_profiles_from(dir.path(), true);
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, "good");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].file, "bad.toml");
    assert!(!dir.path().join("bad.toml").exists());
}

#[test]
fn empty_dir_yields_default_profile() {
    let dir = tempfile::tempdir().unwrap();
    let (profiles, warnings) = load_profiles_from(dir.path(), true);
    assert_eq!(profiles.len(), 1);
    assert!(warnings.is_empty());
}

#[test]
fn profile_with_removed_keys_still_loads() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("old.toml"),
        "id = \"old\"
name = \"Old\"
icon = \"bash\"
shell = \"bash.exe\"
theme = \"dracula\"
font_family = \"Fira Code\"
font_size = 15
",
    )
    .unwrap();
    let (profiles, warnings) = load_profiles_from(dir.path(), true);
    assert!(warnings.is_empty());
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, "old");
    assert_eq!(profiles[0].shell, "bash.exe");
}
