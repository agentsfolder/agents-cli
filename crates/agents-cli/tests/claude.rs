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
fn claude_settings_json_uses_json_field_stamp_and_detects_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, backend: materialize }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [claude] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(&repo.join(".agents/modes/default.md"), "---\nid: default\n---\n\n");
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: { allow: [], deny: [], redact: [\"./.env\"] }\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/claude/adapter.yaml"),
        r#"agentId: claude
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: .claude/settings.json
    format: json
    renderer: { type: template, template: settings.json.hbs }
    writePolicy: { mode: if_generated, gitignore: false }
    driftDetection: { method: sha256, stamp: json_field }
  - path: CLAUDE.md
    format: md
    renderer: { type: template, template: CLAUDE.md.hbs }
    driftDetection: { method: sha256, stamp: comment }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/claude/templates/settings.json.hbs"),
        "{\n  \"permissions\": {\n    \"deny\": [\n      \"Read({{effective.policy.paths.redact.[0]}})\"\n    ]\n  }\n}\n",
    );
    write_file(
        &repo.join(".agents/adapters/claude/templates/CLAUDE.md.hbs"),
        "# CLAUDE\n\nmode={{generation.stamp.mode}}\n",
    );

    let mut sync = Command::cargo_bin("agents").unwrap();
    sync.current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("claude")
        .arg("--backend")
        .arg("materialize");
    sync.assert().success();

    let original = fs::read_to_string(repo.join(".claude/settings.json")).unwrap();
    assert!(original.contains("\"x_generated\""));

    // Drift it.
    fs::write(repo.join(".claude/settings.json"), original.replace("permissions", "perms")).unwrap();

    let mut diff = Command::cargo_bin("agents").unwrap();
    diff.current_dir(repo).arg("diff").arg("--agent").arg("claude");
    diff.assert()
        .success()
        .stdout(predicate::str::contains("CONFLICT(drifted): .claude/settings.json"));
}

#[test]
fn preview_claude_produces_settings_and_claude_md() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [claude] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(&repo.join(".agents/modes/default.md"), "---\nid: default\n---\n\n");
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/claude/adapter.yaml"),
        r#"agentId: claude
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: .claude/settings.json
    format: json
    renderer: { type: template, template: settings.json.hbs }
    driftDetection: { method: sha256, stamp: json_field }
  - path: CLAUDE.md
    format: md
    renderer: { type: template, template: CLAUDE.md.hbs }
    driftDetection: { method: sha256, stamp: comment }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/claude/templates/settings.json.hbs"),
        "{\n  \"ok\": true\n}\n",
    );
    write_file(&repo.join(".agents/adapters/claude/templates/CLAUDE.md.hbs"), "# CLAUDE\n");

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("preview").arg("--agent").arg("claude");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("preview: .claude/settings.json ->"))
        .stdout(predicate::str::contains("preview: CLAUDE.md ->"));
}
