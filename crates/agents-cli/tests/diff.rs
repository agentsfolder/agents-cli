use std::fs;

use predicates::prelude::*;

mod support;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn diff_report_matches_fixture_expectations() {
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
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    write_file(&repo.join("out.md"), "bye\n");

    let mut cmd = support::agents_cmd();
    cmd.current_dir(repo).arg("diff").arg("--agent").arg("a");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("changes:"))
        .stdout(predicate::str::contains("CONFLICT(unmanaged): out.md"));
}
