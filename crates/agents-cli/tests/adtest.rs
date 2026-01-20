use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

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

#[test]
fn agents_test_adapters_failure_shows_unified_diff() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Minimal fixture with an intentionally wrong expected output.
    write_file(
        &root.join("fixtures/basic/repo/.agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe, backend: materialize }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [a] }\n",
    );
    write_file(&root.join("fixtures/basic/repo/.agents/prompts/base.md"), "base\n");
    write_file(
        &root.join("fixtures/basic/repo/.agents/prompts/project.md"),
        "project\n",
    );
    write_file(
        &root.join("fixtures/basic/repo/.agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );
    write_file(
        &root.join("fixtures/basic/repo/.agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
    write_file(
        &root.join("fixtures/basic/repo/.agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: materialize, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n",
    );
    write_file(
        &root.join("fixtures/basic/repo/.agents/adapters/a/templates/t.hbs"),
        "hello\n",
    );

    write_file(&root.join("fixtures/basic/expect/a/out.md"), "bad\n");

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(root)
        .arg("test")
        .arg("adapters")
        .arg("--agent")
        .arg("a");

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("--- expected"))
        .stdout(predicate::str::contains("-bad"))
        .stderr(predicate::str::contains("adapter fixtures failed"));
}
