use std::fs;
use std::path::Path;

use agents_core::fsutil;

#[test]
fn discover_repo_root_prefers_nearest_agents_over_git() {
    let tmp = tempfile::tempdir().unwrap();

    // repo/.git exists
    fs::create_dir_all(tmp.path().join("repo/.git")).unwrap();

    // nested/repo/.agents exists
    fs::create_dir_all(tmp.path().join("repo/nested/.agents")).unwrap();

    let start = tmp.path().join("repo/nested/sub/dir");
    fs::create_dir_all(&start).unwrap();

    let root = fsutil::discover_repo_root(&start).unwrap();
    assert_eq!(root, tmp.path().join("repo/nested"));
}

#[test]
fn repo_relpath_rejects_escape() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();

    let outside = tmp.path().join("outside.txt");
    fs::write(&outside, b"x").unwrap();

    let err = fsutil::repo_relpath(&root, &outside).unwrap_err();
    match err {
        fsutil::FsError::PathEscapesRepo { .. } => {}
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn repo_relpath_noexist_accepts_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();

    let rp = fsutil::repo_relpath_noexist(&root, Path::new("a/b/missing.txt")).unwrap();
    assert_eq!(rp.as_str(), "a/b/missing.txt");
}

#[test]
fn read_to_string_normalizes_crlf() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("file.txt");
    fs::write(&p, b"a\r\nb\r\n").unwrap();

    let s = fsutil::read_to_string(&p).unwrap();
    assert_eq!(s, "a\nb\n");
}

#[test]
fn atomic_write_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("a/b/c.txt");

    fsutil::atomic_write(&p, b"hello").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"hello");

    fsutil::atomic_write(&p, b"world").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"world");
}

#[test]
fn walk_repo_agents_is_deterministic_and_skips_state_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let agents = root.join(".agents");
    fs::create_dir_all(&agents).unwrap();

    // Included
    fs::create_dir_all(agents.join("prompts")).unwrap();
    fs::write(agents.join("prompts/base.md"), b"base").unwrap();

    // State dir should be skipped except .gitignore/state.yaml
    fs::create_dir_all(agents.join("state/logs")).unwrap();
    fs::write(agents.join("state/logs/out.txt"), b"nope").unwrap();
    fs::write(agents.join("state/.gitignore"), b"state.yaml\n").unwrap();

    // Another included file
    fs::create_dir_all(agents.join("policies")).unwrap();
    fs::write(agents.join("policies/default.yaml"), b"id: default").unwrap();

    let first = fsutil::walk_repo_agents(root).unwrap();
    let second = fsutil::walk_repo_agents(root).unwrap();
    assert_eq!(first, second);

    // Ensure skipped file is not present
    for p in &first {
        let rel = p.strip_prefix(root).unwrap();
        assert_ne!(rel, Path::new(".agents/state/logs/out.txt"));
    }
}
