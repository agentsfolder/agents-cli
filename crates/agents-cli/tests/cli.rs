use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_shows_core_commands() {
    let mut cmd = Command::cargo_bin("agents").expect("binary builds");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: agents"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("init"));
}

#[test]
fn validate_in_empty_dir_returns_not_initialized_exit_code_3() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let mut cmd = Command::cargo_bin("agents").expect("binary builds");
    cmd.current_dir(tmp.path()).arg("validate");

    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("repository is not initialized"))
        .stderr(predicate::str::contains("hint: run `agents init`"));
}
