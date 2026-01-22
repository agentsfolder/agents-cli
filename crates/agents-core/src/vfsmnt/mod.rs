use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tempfile::TempDir;
use walkdir::WalkDir;

use crate::fsutil;

const TEMP_PREFIX: &str = "agents-vfsmnt-";
const STALE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone)]
pub struct OverlayFile {
    pub rel_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct VfsMountOptions {
    pub deny_writes: bool,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct VfsMountWorkspace {
    path: PathBuf,
    temp_dir: TempDir,
}

impl VfsMountWorkspace {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn keep(self) -> Result<PathBuf, VfsMountError> {
        Ok(self.temp_dir.keep())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VfsMountError {
    #[error("fs error: {0}")]
    Fs(#[from] fsutil::FsError),

    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("unsupported symlink target: {path}")]
    UnsupportedSymlink { path: PathBuf },
}

pub fn create_workspace(
    repo_root: &Path,
    outputs: &[OverlayFile],
    options: &VfsMountOptions,
) -> Result<VfsMountWorkspace, VfsMountError> {
    cleanup_stale_mounts(options.verbose)?;

    let tmp = fsutil::temp_generation_dir(TEMP_PREFIX)?;
    let path = tmp.path().to_path_buf();

    copy_repo(repo_root, &path)?;
    overlay_outputs(&path, outputs)?;

    if options.deny_writes {
        make_readonly(&path)?;
    }

    Ok(VfsMountWorkspace {
        path,
        temp_dir: tmp,
    })
}

fn cleanup_stale_mounts(verbose: bool) -> Result<(), VfsMountError> {
    let root = std::env::temp_dir();
    let entries = std::fs::read_dir(&root).map_err(|e| VfsMountError::Io {
        path: root.clone(),
        source: e,
    })?;

    let now = SystemTime::now();
    for entry in entries {
        let entry = entry.map_err(|e| VfsMountError::Io {
            path: root.clone(),
            source: e,
        })?;
        let path = entry.path();

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if name.starts_with(TEMP_PREFIX) => name,
            _ => continue,
        };

        let meta = entry.metadata().map_err(|e| VfsMountError::Io {
            path: path.clone(),
            source: e,
        })?;
        let modified = meta.modified().unwrap_or(now);

        if now
            .duration_since(modified)
            .unwrap_or_default()
            .saturating_sub(STALE_TTL)
            > Duration::from_secs(0)
        {
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
                if verbose {
                    eprintln!("vfs_mount: removed stale workspace {}", name);
                }
            }
        }
    }

    Ok(())
}

fn copy_repo(repo_root: &Path, dest_root: &Path) -> Result<(), VfsMountError> {
    for entry in WalkDir::new(repo_root).follow_links(false) {
        let entry = entry.map_err(|e| VfsMountError::Io {
            path: repo_root.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        })?;

        let path = entry.path();
        let rel = path.strip_prefix(repo_root).unwrap_or(path);
        if rel.as_os_str().is_empty() {
            continue;
        }

        let dest = dest_root.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| VfsMountError::Io {
                path: dest.clone(),
                source: e,
            })?;
            continue;
        }

        if entry.file_type().is_symlink() {
            copy_symlink(path, &dest)?;
            continue;
        }

        if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| VfsMountError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
            std::fs::copy(path, &dest).map_err(|e| VfsMountError::Io {
                path: dest.clone(),
                source: e,
            })?;
        }
    }

    Ok(())
}

fn copy_symlink(src: &Path, dest: &Path) -> Result<(), VfsMountError> {
    let target = std::fs::read_link(src).map_err(|e| VfsMountError::Io {
        path: src.to_path_buf(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| VfsMountError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        symlink(&target, dest).map_err(|e| VfsMountError::Io {
            path: dest.to_path_buf(),
            source: e,
        })?;
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        let resolved = if target.is_absolute() {
            target
        } else {
            src.parent().unwrap_or_else(|| Path::new(".")).join(target)
        };
        if resolved.is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| VfsMountError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
            std::fs::copy(&resolved, dest).map_err(|e| VfsMountError::Io {
                path: dest.to_path_buf(),
                source: e,
            })?;
            return Ok(());
        }

        return Err(VfsMountError::UnsupportedSymlink {
            path: src.to_path_buf(),
        });
    }
}

fn overlay_outputs(dest_root: &Path, outputs: &[OverlayFile]) -> Result<(), VfsMountError> {
    for out in outputs {
        let dest = dest_root.join(&out.rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| VfsMountError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fsutil::atomic_write(&dest, &out.bytes)?;
    }

    Ok(())
}

fn make_readonly(root: &Path) -> Result<(), VfsMountError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for entry in WalkDir::new(root).follow_links(false) {
            let entry = entry.map_err(|e| VfsMountError::Io {
                path: root.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;
            let path = entry.path();
            let mut perms = entry
                .metadata()
                .map_err(|e| VfsMountError::Io {
                    path: path.to_path_buf(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                })?
                .permissions();
            let mode = perms.mode() & 0o555;
            perms.set_mode(mode);
            let _ = std::fs::set_permissions(path, perms);
        }
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        let _ = root;
        return Ok(());
    }
}
