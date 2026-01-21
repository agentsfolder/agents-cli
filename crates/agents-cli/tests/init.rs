use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn init_standard_creates_agents_dir_and_validate_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let mut init = Command::cargo_bin("agents").unwrap();
    init.current_dir(root)
        .arg("init")
        .arg("--preset")
        .arg("standard");
    init.assert()
        .success()
        .stdout(predicate::str::contains("ok: initialized .agents/"));

    assert!(root.join(".agents/manifest.yaml").is_file());
    assert!(root.join(".agents/schemas/manifest.schema.json").is_file());
    assert!(root.join(".agents/state/.gitignore").is_file());

    let mut validate = Command::cargo_bin("agents").unwrap();
    validate.current_dir(root).arg("validate");
    validate.assert().success().stdout(predicate::str::contains("ok: schemas valid"));
}

#[test]
fn init_is_deterministic_or_fails_safely_on_second_run() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let mut init1 = Command::cargo_bin("agents").unwrap();
    init1
        .current_dir(root)
        .arg("init")
        .arg("--preset")
        .arg("standard");
    init1.assert().success();

    let mut init2 = Command::cargo_bin("agents").unwrap();
    init2
        .current_dir(root)
        .arg("init")
        .arg("--preset")
        .arg("standard");
    init2
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(".agents is not empty"));
}
