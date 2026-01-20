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
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
}

#[test]
fn sync_then_diff_yields_no_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: materialize, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n    writePolicy: { mode: always }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    write_file(&repo.join("out.md"), "");

    // Sync writes file.
    let mut sync_cmd = Command::cargo_bin("agents").unwrap();
    sync_cmd
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");

    sync_cmd.assert().success();

    // Diff should be noop.
    let mut diff_cmd = Command::cargo_bin("agents").unwrap();
    diff_cmd
        .current_dir(repo)
        .arg("diff")
        .arg("--agent")
        .arg("a");

    diff_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("NOOP: out.md"));
}

#[test]
fn sync_fails_on_unmanaged_file_with_if_generated() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: materialize, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n    writePolicy: { mode: if_generated }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Unmanaged existing file.
    write_file(&repo.join("out.md"), "manual\n");

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unmanaged"));
}
