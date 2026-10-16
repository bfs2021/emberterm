use super::*;

#[test]
fn finds_executable_in_path_list() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fake-shell.exe"), b"").unwrap();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let path_var = format!("/does/not/exist{sep}{}", dir.path().display());
    assert_eq!(
        find_in_path("fake-shell.exe", &path_var),
        Some(dir.path().join("fake-shell.exe"))
    );
}

#[test]
fn missing_executable_is_none() {
    assert_eq!(find_in_path("definitely-not-here.exe", ""), None);
}

#[test]
fn default_shell_is_stable() {
    assert_eq!(default_shell(), default_shell());
}
