use std::fs;

use predicates::prelude::*;

mod support;

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

fn adapter(repo: &std::path::Path, out_path: &str) {
    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        &format!(
            "agentId: a\nversion: '0.1'\nbackendDefaults: {{ preferred: materialize, fallback: materialize }}\noutputs:\n  - path: {}\n    format: md\n    renderer: {{ type: template, template: t.hbs }}\n    driftDetection: {{ method: sha256, stamp: comment }}\n    writePolicy: {{ mode: always }}\n",
            out_path
        ),
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");
}

#[test]
fn deletes_stamped_file() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);
    adapter(repo, "out.md");

    let mut sync_cmd = support::agents_cmd();
    sync_cmd
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");
    sync_cmd.assert().success();
    assert!(repo.join("out.md").is_file());

    let mut clean_cmd = support::agents_cmd();
    clean_cmd
        .current_dir(repo)
        .arg("clean")
        .arg("--agent")
        .arg("a");
    clean_cmd.assert().success();

    assert!(!repo.join("out.md").exists());
}

#[test]
fn does_not_delete_unmanaged_file() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);
    adapter(repo, "out.md");

    write_file(&repo.join("out.md"), "manual\n");
    assert!(repo.join("out.md").is_file());

    let mut clean_cmd = support::agents_cmd();
    clean_cmd
        .current_dir(repo)
        .arg("clean")
        .arg("--agent")
        .arg("a");
    clean_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("skip: out.md"));

    assert!(repo.join("out.md").is_file());
}

#[test]
fn prunes_empty_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);
    adapter(repo, "gen/sub/out.md");

    let mut sync_cmd = support::agents_cmd();
    sync_cmd
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("a")
        .arg("--backend")
        .arg("materialize");
    sync_cmd.assert().success();
    assert!(repo.join("gen/sub/out.md").is_file());

    let mut clean_cmd = support::agents_cmd();
    clean_cmd
        .current_dir(repo)
        .arg("clean")
        .arg("--agent")
        .arg("a");
    clean_cmd.assert().success();

    assert!(!repo.join("gen/sub/out.md").exists());
    assert!(!repo.join("gen/sub").exists());
    assert!(!repo.join("gen").exists());
}
