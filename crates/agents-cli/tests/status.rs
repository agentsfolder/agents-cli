use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn status_output_is_stable_for_minimal_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [] }\n",
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

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("status");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("mode: default"))
        .stdout(predicate::str::contains("policy: safe"))
        .stdout(predicate::str::contains("scopes:"))
        .stdout(predicate::str::contains("skills:"));
}
