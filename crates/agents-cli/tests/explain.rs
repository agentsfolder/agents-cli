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
fn explain_returns_expected_components_after_preview() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
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
        r#"agentId: a
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: AGENTS.md
    format: md
    renderer: { type: template, template: t.hbs }
"#,
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    let mut preview = Command::cargo_bin("agents").unwrap();
    preview
        .current_dir(repo)
        .arg("preview")
        .arg("--agent")
        .arg("a");
    preview.assert().success();

    let mut explain = Command::cargo_bin("agents").unwrap();
    explain
        .current_dir(repo)
        .arg("explain")
        .arg("AGENTS.md");

    explain
        .assert()
        .success()
        .stdout(predicate::str::contains("path: AGENTS.md"))
        .stdout(predicate::str::contains("adapter: a"))
        .stdout(predicate::str::contains("template: t.hbs"))
        .stdout(predicate::str::contains("prompt_sources:"));
}

#[test]
fn explain_reports_unmanaged_file_helpfully() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(&repo.join("notes.txt"), "hello\n");

    let mut explain = Command::cargo_bin("agents").unwrap();
    explain.current_dir(repo).arg("explain").arg("notes.txt");

    explain
        .assert()
        .failure()
        .stderr(predicate::str::contains("unmanaged file"));
}
