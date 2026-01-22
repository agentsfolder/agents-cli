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
fn cursor_rules_are_deterministic_and_diff_is_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, backend: materialize }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [cursor] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\nmode\n",
    );
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: { allow: [\"src/**\"], deny: [\"secrets/**\"], redact: [\"secrets/**\"] }\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/cursor/adapter.yaml"),
        r#"agentId: cursor
version: '0.1'
backendDefaults: { preferred: materialize, fallback: materialize }
outputs:
  - path: .cursor/rules/00-current-mode.md
    format: md
    renderer: { type: template, template: 00-current-mode.md.hbs }
    writePolicy: { mode: if_generated, gitignore: false }
    driftDetection: { method: sha256, stamp: comment }
  - path: .cursor/rules/10-guidance.md
    format: md
    renderer: { type: template, template: 10-guidance.md.hbs }
    writePolicy: { mode: if_generated, gitignore: false }
    driftDetection: { method: sha256, stamp: comment }
  - path: .cursor/rules/20-policy.md
    format: md
    renderer: { type: template, template: 20-policy.md.hbs }
    writePolicy: { mode: if_generated, gitignore: false }
    driftDetection: { method: sha256, stamp: comment }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/cursor/templates/00-current-mode.md.hbs"),
        "# Current Mode\n\nmode={{generation.stamp.mode}}\n",
    );
    write_file(
        &repo.join(".agents/adapters/cursor/templates/10-guidance.md.hbs"),
        "# Guidance\n\n{{effective.prompts.composed_md}}\n",
    );
    write_file(
        &repo.join(".agents/adapters/cursor/templates/20-policy.md.hbs"),
        "# Policy\n\nallow={{join effective.policy.paths.allow \",\"}}\n",
    );

    let mut sync1 = support::agents_cmd();
    sync1
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("cursor");
    sync1.assert().success();

    let a0 = fs::read(repo.join(".cursor/rules/00-current-mode.md")).unwrap();
    let a1 = fs::read(repo.join(".cursor/rules/10-guidance.md")).unwrap();
    let a2 = fs::read(repo.join(".cursor/rules/20-policy.md")).unwrap();

    let mut sync2 = support::agents_cmd();
    sync2
        .current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("cursor");
    sync2.assert().success();

    let b0 = fs::read(repo.join(".cursor/rules/00-current-mode.md")).unwrap();
    let b1 = fs::read(repo.join(".cursor/rules/10-guidance.md")).unwrap();
    let b2 = fs::read(repo.join(".cursor/rules/20-policy.md")).unwrap();

    assert_eq!(a0, b0);
    assert_eq!(a1, b1);
    assert_eq!(a2, b2);

    let mut diff = support::agents_cmd();
    diff.current_dir(repo)
        .arg("diff")
        .arg("--agent")
        .arg("cursor");
    diff.assert()
        .success()
        .stdout(predicate::str::contains("noop=3"))
        .stdout(predicate::str::contains(
            "NOOP: .cursor/rules/00-current-mode.md",
        ))
        .stdout(predicate::str::contains(
            "NOOP: .cursor/rules/10-guidance.md",
        ))
        .stdout(predicate::str::contains("NOOP: .cursor/rules/20-policy.md"));
}
