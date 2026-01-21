use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::vfsctr::docker::{DockerError, DockerRuntime};

/// Minimal v1 `vfs_container` backend invocation.
///
/// Runtime contract:
/// - Host repo is mounted read-only at `/__agents_repo`.
/// - Generated outputs are mounted read-only at `/__agents_out`.
/// - Container creates a writable `/workspace` by copying the repo contents.
/// - Container then overlays generated outputs into `/workspace`.
/// - Agent command is executed with CWD `/workspace`.
#[derive(Debug, Clone)]
pub struct VfsContainerInvocation {
    pub repo_root: PathBuf,
    pub outputs_dir: PathBuf,
    pub image: String,
    pub cmd: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub verbose: bool,

    /// Best-effort: disable network access by using docker's `--network none`.
    pub deny_network: bool,

    /// Best-effort: make `/workspace` read-only inside the container (chmod -R a-w).
    pub deny_writes: bool,
}

impl VfsContainerInvocation {
    pub fn docker_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-i".to_string(),
            "--workdir".to_string(),
            "/workspace".to_string(),
            // Host repo and outputs are mounted read-only.
            "--mount".to_string(),
            format!(
                "type=bind,source={},target=/__agents_repo,readonly",
                self.repo_root.display()
            ),
            "--mount".to_string(),
            format!(
                "type=bind,source={},target=/__agents_out,readonly",
                self.outputs_dir.display()
            ),
        ];

        if self.deny_network {
            args.push("--network".to_string());
            args.push("none".to_string());
        }

        for (k, v) in &self.env {
            args.push("-e".to_string());
            args.push(format!("{k}={v}"));
        }

        args.push(self.image.clone());

        // Use /bin/sh for maximum compatibility.
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push(entry_script(self.verbose, self.deny_writes));
        args.push("--".to_string());

        args.extend(self.cmd.clone());
        args
    }

    pub fn run(&self, docker: &DockerRuntime) -> Result<std::process::Output, DockerError> {
        docker.check_available()?;
        docker.check_daemon()?;

        let args = self.docker_args();
        docker.run(&args)
    }
}

pub fn default_image() -> String {
    std::env::var("AGENTS_VFSCTR_IMAGE").unwrap_or_else(|_| "alpine:3.19".to_string())
}

fn entry_script(verbose: bool, deny_writes: bool) -> String {
    // Use tar to preserve file modes and create nested directories.
    // Avoid bashisms: run under /bin/sh.
    let mut s = String::new();
    s.push_str("set -eu\n");
    s.push_str("mkdir -p /workspace\n");

    if verbose {
        s.push_str("echo 'agents vfs_container: copying repo to /workspace' 1>&2\n");
    }
    s.push_str("cd /__agents_repo\n");
    s.push_str("tar cf - . | (cd /workspace && tar xpf -)\n");

    if verbose {
        s.push_str("echo 'agents vfs_container: overlaying generated outputs' 1>&2\n");
    }
    s.push_str("cd /__agents_out\n");
    s.push_str("tar cf - . | (cd /workspace && tar xpf -)\n");

    if deny_writes {
        if verbose {
            s.push_str("echo 'agents vfs_container: disabling writes in /workspace' 1>&2\n");
        }
        // Best-effort; if chmod fails for any reason, proceed.
        s.push_str("chmod -R a-w /workspace || true\n");
    }

    if verbose {
        s.push_str("echo 'agents vfs_container: exec' 1>&2\n");
    }

    // $@ is the agent command.
    s.push_str("cd /workspace\n");
    s.push_str("exec \"$@\"\n");
    s
}

pub fn normalize_repo_root(p: &Path) -> PathBuf {
    // Keep as-is for v1; caller is responsible for providing a valid repo root.
    p.to_path_buf()
}
