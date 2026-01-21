use std::process::{Command, Output};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("docker CLI not found")]
    NotInstalled,

    #[error("docker daemon not reachable")]
    DaemonUnavailable { stdout: String, stderr: String },

    #[error("docker command failed: {message}")]
    Failed {
        message: String,
        stdout: String,
        stderr: String,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct DockerRuntime {
    docker_bin: String,
}

impl DockerRuntime {
    pub fn new() -> Self {
        Self {
            docker_bin: "docker".to_string(),
        }
    }

    pub fn check_available(&self) -> Result<(), DockerError> {
        let out = Command::new(&self.docker_bin).arg("--version").output();

        match out {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(DockerError::Failed {
                message: "docker --version returned non-zero exit status".to_string(),
                stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                stderr: String::from_utf8_lossy(&o.stderr).to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(DockerError::NotInstalled),
            Err(e) => Err(DockerError::Io(e)),
        }
    }

    pub fn check_daemon(&self) -> Result<(), DockerError> {
        // `docker info` is the simplest cross-platform daemon readiness check.
        let out = Command::new(&self.docker_bin).arg("info").output();

        match out {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(DockerError::DaemonUnavailable {
                stdout: String::from_utf8_lossy(&o.stdout).to_string(),
                stderr: String::from_utf8_lossy(&o.stderr).to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(DockerError::NotInstalled),
            Err(e) => Err(DockerError::Io(e)),
        }
    }

    pub fn run(&self, args: &[String]) -> Result<Output, DockerError> {
        let out = Command::new(&self.docker_bin).args(args).output()?;
        if out.status.success() {
            return Ok(out);
        }

        Err(DockerError::Failed {
            message: format!("docker {} returned non-zero exit status", args.join(" ")),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        })
    }
}

impl Default for DockerRuntime {
    fn default() -> Self {
        Self::new()
    }
}
