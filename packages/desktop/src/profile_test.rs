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

#[test]
fn profile_path_rejects_ids_that_escape_the_folder() {
    let dir = Path::new("profiles");
    for bad in [
        "",
        "..",
        "../x",
        r"..\x",
        "a/b",
        r"a\b",
        "C:x",
        "x.toml",
        &"a".repeat(65),
    ] {
        assert!(profile_path_in(dir, bad).is_err(), "accepted {bad:?}");
    }
}

#[test]
fn profile_path_accepts_generated_and_builtin_ids() {
    let dir = Path::new("profiles");
    for ok in ["default", "3f9a1c2e", "builtin-gitbash", "my_profile-2"] {
        assert_eq!(
            profile_path_in(dir, ok).unwrap(),
            dir.join(format!("{ok}.toml"))
        );
    }
}

#[test]
fn save_and_delete_refuse_a_traversal_id() {
    let p = Profile {
        id: "../evil".into(),
        ..Profile::default()
    };
    assert!(save_profile(&p).is_err());
    assert!(delete_profile("../evil").is_err());
}
