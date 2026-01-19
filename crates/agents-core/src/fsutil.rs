use std::fmt;
use std::io;
use std::path::PathBuf;

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
