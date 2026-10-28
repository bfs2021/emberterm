use super::*;

#[test]
fn network_paths_are_refused() {
    for bad in [
        r"\host\share\repo",
        "//host/share/repo",
        r"\?\UNC\host\share",
    ] {
        assert_eq!(safe_cwd(bad), None, "accepted {bad:?}");
    }
}

#[test]
fn missing_and_file_paths_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f.txt");
    std::fs::write(&file, "x").unwrap();
    assert_eq!(safe_cwd(file.to_str().unwrap()), None);
    assert_eq!(safe_cwd(dir.path().join("nope").to_str().unwrap()), None);
}

#[test]
fn an_existing_local_folder_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_str().unwrap();
    assert_eq!(safe_cwd(p), Some(p));
}

#[test]
fn git_runs_without_repo_controlled_programs_or_locks() {
    let cmd = command(".");
    let args: Vec<_> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, ["-c", "core.fsmonitor=false", "--no-optional-locks"]);
    let env: Vec<_> = cmd.get_envs().collect();
    assert!(env
        .iter()
        .any(|(k, v)| *k == "GIT_OPTIONAL_LOCKS" && *v == Some("0".as_ref())));
}
