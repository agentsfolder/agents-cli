use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

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

    let repo_root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();
    let fixture_script = repo_root.join("fixtures/runner/dummy-agent.sh");
    let agent_path = repo.join("dummy-agent.sh");
    fs::copy(&fixture_script, &agent_path).unwrap();
    make_executable(&agent_path);

    let mut cmd = Command::cargo_bin("agents").unwrap();
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
