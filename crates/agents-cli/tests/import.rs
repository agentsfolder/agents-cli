use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn import_copilot_creates_agents_and_validates() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(
        &root.join(".github/copilot-instructions.md"),
        "# Copilot\n\nDo the thing.\n",
    );

    let mut import = Command::cargo_bin("agents").unwrap();
    import
        .current_dir(root)
        .arg("import")
        .arg("--from")
        .arg("copilot");
    import
        .assert()
        .success()
        .stdout(predicate::str::contains("ok: imported into .agents/"));

    assert!(root.join(".agents/manifest.yaml").is_file());
    assert!(root.join(".agents/prompts/snippets/copilot.md").is_file());
    assert!(root.join(".agents/modes/copilot-import.md").is_file());

    let mut validate = Command::cargo_bin("agents").unwrap();
    validate.current_dir(root).arg("validate");
    validate.assert().success().stdout(predicate::str::contains("ok: schemas valid"));
}

#[test]
fn import_dry_run_does_not_write_anything() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write_file(&root.join(".github/copilot-instructions.md"), "x\n");

    let mut import = Command::cargo_bin("agents").unwrap();
    import
        .current_dir(root)
        .arg("import")
        .arg("--from")
        .arg("copilot")
        .arg("--dry-run");
    import
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run: would write"));

    assert!(!root.join(".agents").exists());
}
