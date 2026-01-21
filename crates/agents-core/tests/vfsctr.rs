use std::collections::BTreeMap;
use std::path::PathBuf;

use agents_core::vfsctr::docker::DockerRuntime;
use agents_core::vfsctr::run::{default_image, VfsContainerInvocation};

#[test]
fn docker_args_are_deterministic() {
    let repo_root = PathBuf::from("/repo");
    let outputs_dir = PathBuf::from("/out");

    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("Z".to_string(), "z".to_string());
    env.insert("A".to_string(), "a".to_string());

    let inv = VfsContainerInvocation {
        repo_root,
        outputs_dir,
        image: "alpine:3.19".to_string(),
        cmd: vec!["sh".to_string(), "-c".to_string(), "echo ok".to_string()],
        env,
        verbose: false,
        deny_network: false,
        deny_writes: false,
    };

    let args1 = inv.docker_args();
    let args2 = inv.docker_args();
    assert_eq!(args1, args2);

    // Env vars should be in stable sorted order (BTreeMap).
    let joined = args1.join("\n");
    let idx_a = joined.find("A=a").unwrap();
    let idx_z = joined.find("Z=z").unwrap();
    assert!(idx_a < idx_z);
}

#[test]
fn entry_script_includes_copy_overlay_and_exec() {
    let inv = VfsContainerInvocation {
        repo_root: PathBuf::from("/repo"),
        outputs_dir: PathBuf::from("/out"),
        image: "alpine:3.19".to_string(),
        cmd: vec!["true".to_string()],
        env: BTreeMap::new(),
        verbose: true,
        deny_network: false,
        deny_writes: true,
    };

    let args = inv.docker_args();
    let pos = args.iter().position(|a| a == "-c").unwrap();
    let script = &args[pos + 1];
    assert!(script.contains("tar cf - ."));
    assert!(script.contains("/__agents_out"));
    assert!(script.contains("chmod -R a-w /workspace"));
    assert!(script.contains("exec \"$@\""));
}

#[test]
fn docker_integration_overlay_works_when_enabled() {
    if std::env::var("AGENTS_DOCKER_TESTS").is_err() {
        return;
    }

    let repo = tempfile::tempdir().unwrap();
    std::fs::write(repo.path().join("repo.txt"), "repo\n").unwrap();

    let out = tempfile::tempdir().unwrap();
    std::fs::write(out.path().join("generated.txt"), "generated\n").unwrap();

    let inv = VfsContainerInvocation {
        repo_root: repo.path().to_path_buf(),
        outputs_dir: out.path().to_path_buf(),
        image: default_image(),
        cmd: vec![
            "sh".to_string(),
            "-c".to_string(),
            "test -f generated.txt && cat generated.txt".to_string(),
        ],
        env: BTreeMap::new(),
        verbose: true,
        deny_network: true,
        deny_writes: false,
    };

    let docker = DockerRuntime::new();
    let res = inv.run(&docker);
    let out = match res {
        Ok(o) => o,
        Err(e) => panic!("docker integration failed: {e}"),
    };

    assert_eq!(String::from_utf8_lossy(&out.stdout), "generated\n");
}
