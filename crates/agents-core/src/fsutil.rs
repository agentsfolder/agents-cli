use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};

use walkdir::WalkDir;

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

/// A repo-relative path normalized to use `/` separators.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoPath(String);

impl RepoPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Convert `path` to a repo-relative normalized `RepoPath`.
///
/// This rejects paths that escape the repo root.
pub fn repo_relpath(root: &Path, path: &Path) -> FsResult<RepoPath> {
    let root_abs = root.canonicalize().map_err(|e| FsError::Io {
        path: root.to_path_buf(),
        source: e,
    })?;

    let candidate_abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_abs.join(path)
    };

    let candidate_abs = candidate_abs.canonicalize().map_err(|e| FsError::Io {
        path: candidate_abs.clone(),
        source: e,
    })?;

    let rel = candidate_abs
        .strip_prefix(&root_abs)
        .map_err(|_| FsError::PathEscapesRepo {
            root: root_abs.clone(),
            path: candidate_abs.clone(),
        })?;

    Ok(RepoPath(path_to_forward_slash(rel)))
}

fn path_to_forward_slash(path: &Path) -> String {
    let mut out = String::new();
    for comp in path.components() {
        if !out.is_empty() {
            out.push('/');
        }

        match comp {
            Component::Normal(os) => out.push_str(&os.to_string_lossy()),
            Component::CurDir => out.push('.'),
            Component::ParentDir => out.push_str(".."),
            Component::RootDir => {}
            Component::Prefix(prefix) => out.push_str(&prefix.as_os_str().to_string_lossy()),
        }
    }

    out
}

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

pub fn display_repo_path(root: &Path, path: &Path) -> FsResult<String> {
    repo_relpath(root, path).map(|p| p.to_string())
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

pub fn walk_repo_agents(root: &Path) -> FsResult<Vec<PathBuf>> {
    let agents_root = agents_dir(root);

    let mut paths = vec![];
    for entry in WalkDir::new(&agents_root).follow_links(false) {
        let entry = entry.map_err(|e| FsError::Io {
            path: agents_root.clone(),
            source: io::Error::new(io::ErrorKind::Other, e.to_string()),
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Skip `.agents/state/**` except `.agents/state/.gitignore` and optional `state.yaml`.
        if let Ok(rel) = path.strip_prefix(&agents_root) {
            let is_state = rel
                .components()
                .next()
                .is_some_and(|c| c == Component::Normal("state".as_ref()));

            if is_state {
                let keep =
                    rel == Path::new("state/.gitignore") || rel == Path::new("state/state.yaml");
                if !keep {
                    continue;
                }
            }
        }

        paths.push(path.to_path_buf());
    }

    paths.sort_by(|a, b| path_to_forward_slash(a).cmp(&path_to_forward_slash(b)));
    Ok(paths)
}
