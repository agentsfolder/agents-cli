use std::path::PathBuf;

use assert_cmd::Command;

#[test]
fn agents_test_adapters_passes_on_basic_fixture() {
    let repo_root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(&repo_root)
        .arg("test")
        .arg("adapters")
        .arg("--agent")
        .arg("a");

    cmd.assert().success();
}
