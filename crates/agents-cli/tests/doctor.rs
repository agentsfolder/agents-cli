use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn base_repo(repo: &std::path::Path) {
    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe, backend: materialize }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [a] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(&repo.join(".agents/modes/default.md"), "---\nid: default\n---\n\n");
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: materialize, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n    writePolicy: { mode: always }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");
}

#[test]
fn doctor_passes_on_clean_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);

    // Make repo clean by syncing first.
    let mut sync_cmd = Command::cargo_bin("agents").unwrap();
    sync_cmd
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");
    sync_cmd.assert().success();

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("doctor: errors=0"));
}

#[test]
fn doctor_reports_drift_when_generated_file_is_edited() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);

    let mut sync_cmd = Command::cargo_bin("agents").unwrap();
    sync_cmd
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");
    sync_cmd.assert().success();

    // Drift it (edit generated file).
    let mut s = fs::read_to_string(repo.join("out.md")).unwrap();
    s.push_str("\nmanual edit\n");
    fs::write(repo.join("out.md"), s).unwrap();

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("generated files drifted"));
}

#[test]
fn doctor_fix_creates_missing_state_gitignore() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);

    assert!(!repo.join(".agents/state/.gitignore").exists());

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("doctor").arg("--fix");
    cmd.assert().success();

    let content = fs::read_to_string(repo.join(".agents/state/.gitignore")).unwrap();
    assert!(content.contains("state.yaml"));
}
