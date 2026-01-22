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
defaults: { mode: default, policy: safe, sharedSurfacesOwner: core }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [core] }\n",
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
fn preview_core_emits_agents_md() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);

    let mut cmd = support::agents_cmd();
    cmd.current_dir(repo)
        .arg("preview")
        .arg("--agent")
        .arg("core");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("preview: AGENTS.md ->"));
}

#[test]
fn sync_core_is_deterministic() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    base_repo(repo);

    let mut sync1 = support::agents_cmd();
    sync1
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("core");
    sync1.assert().success();

    let a = fs::read(repo.join("AGENTS.md")).unwrap();

    let mut sync2 = support::agents_cmd();
    sync2
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("core");
    sync2.assert().success();

    let b = fs::read(repo.join("AGENTS.md")).unwrap();
    assert_eq!(a, b);
}
