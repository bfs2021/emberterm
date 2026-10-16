use super::*;

#[test]
fn write_atomic_creates_and_replaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.toml");
    write_atomic(&path, b"one").unwrap();
    write_atomic(&path, b"two").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"two");
}

#[test]
fn write_atomic_creates_missing_parent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("a.json");
    write_atomic(&path, b"x").unwrap();
    assert!(path.exists());
}

#[test]
fn write_atomic_leaves_no_tmp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.toml");
    write_atomic(&path, b"x").unwrap();
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("a.toml")]);
}

#[test]
fn write_atomic_failure_cleans_up_tmp() {
    let dir = tempfile::tempdir().unwrap();
    // Renaming a file over a non-empty directory fails on every platform.
    let target = dir.path().join("occupied");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("keep"), b"k").unwrap();
    assert!(write_atomic(&target, b"x").is_err());
    assert!(!dir.path().join("occupied.tmp").exists());
    assert!(target.join("keep").exists());
}

#[test]
fn quarantine_moves_file_aside() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, b"broken = [").unwrap();
    let backup = quarantine(&path).unwrap();
    assert!(!path.exists());
    assert_eq!(std::fs::read(&backup).unwrap(), b"broken = [");
    let name = backup.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("config.toml.broken-"), "{name}");
}
