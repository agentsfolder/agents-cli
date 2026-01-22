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
fn preview_copilot_produces_repo_and_scope_instruction_files() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [copilot] }\n",
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
        &repo.join(".agents/scopes/api.yaml"),
        "id: api.v2\napplyTo: [\"packages/api/**\"]\npriority: 0\noverrides: {}\n",
    );
    write_file(
        &repo.join(".agents/scopes/web.yaml"),
        "id: web\napplyTo: [\"packages/web/**\"]\npriority: 0\noverrides: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/copilot/adapter.yaml"),
        r#"agentId: copilot
version: '0.1'
backendDefaults: { preferred: materialize, fallback: materialize }
outputs:
  - path: .github/copilot-instructions.md
    format: md
    renderer: { type: template, template: copilot-instructions.md.hbs }
    driftDetection: { method: sha256, stamp: comment }
  - path: .github/instructions/{{scopeId}}.instructions.md
    format: md
    renderer: { type: template, template: scope.instructions.md.hbs }
    driftDetection: { method: sha256, stamp: frontmatter }
"#,
    );
    write_file(
        &repo.join(".agents/adapters/copilot/templates/copilot-instructions.md.hbs"),
        "# Copilot Instructions\n",
    );
    write_file(
        &repo.join(".agents/adapters/copilot/templates/scope.instructions.md.hbs"),
        "---\napplyTo: \"{{join scope.applyTo \",\"}}\"\n---\n\n# Scope {{scope.id}}\n",
    );

    let mut cmd = support::agents_cmd();
    cmd.current_dir(repo)
        .arg("preview")
        .arg("--agent")
        .arg("copilot");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "preview: .github/copilot-instructions.md ->",
        ))
        .stdout(predicate::str::contains(
            "preview: .github/instructions/api_v2.instructions.md ->",
        ))
        .stdout(predicate::str::contains(
            "preview: .github/instructions/web.instructions.md ->",
        ));
}
