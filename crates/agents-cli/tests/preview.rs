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
fn preview_produces_expected_paths_in_output() {
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
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    write_file(&repo.join("out.md"), "");

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("preview").arg("--agent").arg("a");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("preview: out.md ->"));
}

#[test]
fn preview_fails_on_missing_template_vars() {
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
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n",
    );
    write_file(
        &repo.join(".agents/adapters/a/templates/t.hbs"),
        "missing: {{missing.value}}\n",
    );

    let mut cmd = Command::cargo_bin("agents").unwrap();
    cmd.current_dir(repo).arg("preview").arg("--agent").arg("a");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("template render error"));
}

#[test]
fn preview_renders_composed_prompt_in_templates() {
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
    write_file(&repo.join(".agents/prompts/snippets/extra.md"), "snippet\n");
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\nincludeSnippets: [extra]\n---\n\n",
    );
    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n",
    );
    write_file(
        &repo.join(".agents/adapters/a/templates/t.hbs"),
        "{{effective.prompts.composed_md}}",
    );

    let output = Command::cargo_bin("agents")
        .unwrap()
        .current_dir(repo)
        .arg("preview")
        .arg("--agent")
        .arg("a")
        .arg("--keep-temp")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .find(|l| l.starts_with("preview: out.md -> "))
        .expect("preview output path");
    let dest = line
        .split("preview: out.md -> ")
        .nth(1)
        .expect("temp output path");
    let dest = dest.trim();

    let temp_line = stdout
        .lines()
        .find(|l| l.starts_with("temp: "))
        .expect("temp dir output");
    let temp_dir = temp_line
        .split("temp: ")
        .nth(1)
        .expect("temp dir path")
        .trim();

    let rendered = fs::read_to_string(dest).unwrap();
    assert!(rendered.contains("base\n\nproject\n\nsnippet\n"));

    fs::remove_dir_all(temp_dir).unwrap();
}
