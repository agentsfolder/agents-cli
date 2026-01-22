use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

#[test]
fn run_executes_dummy_agent_with_passthrough_args() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [dummy] }\n",
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
        &repo.join(".agents/adapters/dummy/adapter.yaml"),
        "agentId: dummy\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: out.md.hbs }\n",
    );
    write_file(
        &repo.join(".agents/adapters/dummy/templates/out.md.hbs"),
        "output\n",
    );

    let fixture_script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/assets/dummy-agent.sh");
    let agent_path = repo.join("dummy-agent.sh");
    fs::copy(&fixture_script, &agent_path).unwrap();
    make_executable(&agent_path);

    let mut cmd = support::agents_cmd();
    cmd.current_dir(repo)
        .arg("run")
        .arg("./dummy-agent.sh")
        .arg("--adapter")
        .arg("dummy")
        .arg("--backend")
        .arg("materialize")
        .arg("--")
        .arg("--flag")
        .arg("value");

    cmd.assert().code(42);

    let args = fs::read_to_string(repo.join("run-args.txt")).unwrap();
    assert_eq!(args, "--flag\nvalue\n");
}

#[test]
fn run_executes_in_vfs_mount_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
 defaults: { mode: default, policy: safe }\n\
 enabled: { modes: [default], policies: [safe], skills: [], adapters: [dummy] }\n",
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
        &repo.join(".agents/adapters/dummy/adapter.yaml"),
        "agentId: dummy\nversion: '0.1'\nbackendDefaults: { preferred: vfs_mount, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: out.md.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n    writePolicy: { mode: always }\n",
    );
    write_file(
        &repo.join(".agents/adapters/dummy/templates/out.md.hbs"),
        "output\n",
    );

    write_file(&repo.join("out.md"), "repo\n");

    let agent_path = repo.join("dummy-agent.sh");
    write_file(
        &agent_path,
        "#!/bin/sh\nset -eu\nresult=\"$1\"\ncat out.md > \"$result\"\n",
    );
    make_executable(&agent_path);

    let result_path = repo.join("run-result.txt");
    let mut cmd = support::agents_cmd();
    cmd.current_dir(repo)
        .arg("run")
        .arg("./dummy-agent.sh")
        .arg("--adapter")
        .arg("dummy")
        .arg("--backend")
        .arg("vfs-mount")
        .arg("--")
        .arg(result_path.to_string_lossy().to_string());

    cmd.assert().success();

    let result = fs::read_to_string(&result_path).unwrap();
    assert!(result.contains("output"));
    assert_eq!(fs::read_to_string(repo.join("out.md")).unwrap(), "repo\n");
}
