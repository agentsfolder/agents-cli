use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum FsError {
    Io { path: PathBuf, source: io::Error },
    PathEscapesRepo { root: PathBuf, path: PathBuf },
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::Io { path, source } => {
                write!(f, "io error at {}: {}", path.display(), source)
            }
            FsError::PathEscapesRepo { root, path } => write!(
                f,
                "path escapes repo root (root={}, path={})",
                root.display(),
                path.display()
            ),
        }
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FsError::Io { source, .. } => Some(source),
            FsError::PathEscapesRepo { .. } => None,
        }
    }
}

pub type FsResult<T> = Result<T, FsError>;

pub fn agents_dir(root: &Path) -> PathBuf {
    root.join(".agents")
}

pub fn discover_repo_root(start: &Path) -> FsResult<PathBuf> {
    let mut cur = start;
    let mut best_git: Option<PathBuf> = None;

    loop {
        if agents_dir(cur).is_dir() {
            return Ok(cur.to_path_buf());
        }

        if cur.join(".git").is_dir() && best_git.is_none() {
            best_git = Some(cur.to_path_buf());
        }

        match cur.parent() {
            Some(parent) => cur = parent,
            None => break,
        }
    }

    if let Some(git) = best_git {
        Ok(git)
    } else {
        Ok(start.to_path_buf())
    }
}

pub fn require_agents_dir(root: &Path) -> FsResult<()> {
    let dir = agents_dir(root);
    if dir.is_dir() {
        Ok(())
    } else {
        Err(FsError::Io {
            path: dir,
            source: io::Error::new(io::ErrorKind::NotFound, ".agents/ not found"),
        })
    }
}
