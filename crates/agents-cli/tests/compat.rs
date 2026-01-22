use std::fs;

mod support;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn compat_output_is_stable_for_multiple_adapters() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [b, a] }\n",
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
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: AGENTS.md\n    format: md\n    surface: shared:AGENTS.md\n    renderer: { type: template, template: a.hbs }\n  - path: out.md\n    format: md\n    renderer: { type: template, template: out.hbs }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/a.hbs"), "a\n");
    write_file(&repo.join(".agents/adapters/a/templates/out.hbs"), "out\n");

    write_file(
        &repo.join(".agents/adapters/b/adapter.yaml"),
        "agentId: b\nversion: '0.1'\nbackendDefaults: { preferred: materialize, fallback: materialize }\ncapabilityMapping: { exec: advisory }\noutputs:\n  - path: config.jsonc\n    format: jsonc\n    renderer: { type: template, template: b.hbs }\n",
    );
    write_file(&repo.join(".agents/adapters/b/templates/b.hbs"), "b\n");

    let mut cmd = support::agents_cmd();
    let output = cmd.current_dir(repo).arg("compat").output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "agent: a\n\
outputs: AGENTS.md, out.md\n\
surfaces: shared:AGENTS.md\n\
backend: preferred VfsContainer, fallback Materialize\n\
policy_mapping: advisory\n\
enforcement: filesystem=enforced via read-only mounts, network=best-effort (container networking), exec=limited (advisory allow/deny)\n\
limitations:\n\
- requires container runtime for vfs_container\n\
\n\
agent: b\n\
outputs: config.jsonc\n\
surfaces: <none>\n\
backend: preferred Materialize, fallback Materialize\n\
policy_mapping: custom (capabilityMapping)\n\
enforcement: filesystem=not enforced (writes to repo), network=advisory, exec=advisory\n\
limitations:\n\
- writes generated outputs into the repo\n";
    assert_eq!(stdout, expected);
}
