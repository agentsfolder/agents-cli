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
fn opencode_jsonc_uses_json_field_stamp_and_detects_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, backend: materialize, sharedSurfacesOwner: opencode }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [opencode] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(&repo.join(".agents/modes/default.md"), "---\nid: default\n---\n\n");
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/opencode/adapter.yaml"),
        r#"agentId: opencode
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: opencode.jsonc
    format: jsonc
    renderer: { type: template, template: opencode.jsonc.hbs }
    driftDetection: { method: sha256, stamp: json_field }
  - path: AGENTS.md
    format: md
    surface: shared:AGENTS.md
    collision: shared_owner
    renderer: { type: template, template: AGENTS.md.hbs }
    driftDetection: { method: sha256, stamp: comment }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/opencode/templates/opencode.jsonc.hbs"),
        "{\n  \"$schema\": \"https://opencode.ai/config.json\"\n}\n",
    );
    write_file(&repo.join(".agents/adapters/opencode/templates/AGENTS.md.hbs"), "# AGENTS\n");

    let mut sync = Command::cargo_bin("agents").unwrap();
    sync.current_dir(repo)
        .arg("sync")
        .arg("--agent")
        .arg("opencode")
        .arg("--backend")
        .arg("materialize");
    sync.assert().success();

    let original = fs::read_to_string(repo.join("opencode.jsonc")).unwrap();
    assert!(original.contains("\"x_generated\""));

    // Drift it (edit generated file).
    let edited = original.replace("https://opencode.ai/config.json", "https://example.invalid");
    fs::write(repo.join("opencode.jsonc"), edited).unwrap();

    let mut diff = Command::cargo_bin("agents").unwrap();
    diff.current_dir(repo)
        .arg("diff")
        .arg("--agent")
        .arg("opencode");
    diff.assert()
        .success()
        .stdout(predicate::str::contains("CONFLICT(drifted): opencode.jsonc"));
}

#[test]
fn preview_opencode_produces_jsonc_and_agents_md() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, sharedSurfacesOwner: opencode }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [opencode] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(&repo.join(".agents/modes/default.md"), "---\nid: default\n---\n\n");
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/opencode/adapter.yaml"),
        r#"agentId: opencode
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: opencode.jsonc
    format: jsonc
    renderer: { type: template, template: opencode.jsonc.hbs }
    driftDetection: { method: sha256, stamp: json_field }
  - path: AGENTS.md
    format: md
    surface: shared:AGENTS.md
    collision: shared_owner
    renderer: { type: template, template: AGENTS.md.hbs }
    driftDetection: { method: sha256, stamp: comment }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/opencode/templates/opencode.jsonc.hbs"),
        "{\n  \"$schema\": \"https://opencode.ai/config.json\"\n}\n",
    );
    write_file(&repo.join(".agents/adapters/opencode/templates/AGENTS.md.hbs"), "# AGENTS\n");

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("preview").arg("--agent").arg("opencode");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("preview: AGENTS.md ->"))
        .stdout(predicate::str::contains("preview: opencode.jsonc ->"));
}
