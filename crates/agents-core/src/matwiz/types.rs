use std::path::{Path, PathBuf};

use crate::fsutil::RepoPath;
use crate::outputs::OutputPlan;
use crate::stamps::{DriftStatus, StampMeta};

#[derive(Debug, Clone)]
pub struct RenderedOutput {
    pub path: RepoPath,

    /// Fully rendered, stamped bytes to write to disk.
    pub bytes: Vec<u8>,

    pub stamp_meta: StampMeta,

    pub drift_status: DriftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictReason {
    Unmanaged,
    Drifted,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ConflictDetail {
    pub path: RepoPath,
    pub reason: ConflictReason,
    pub message: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ApplyReport {
    pub written: Vec<RepoPath>,
    pub skipped: Vec<RepoPath>,
    pub conflicts: Vec<RepoPath>,

    #[allow(dead_code)]
    pub conflict_details: Vec<ConflictDetail>,
}

#[derive(Debug, Clone)]
pub struct BackendSession {
    pub repo_root: PathBuf,
    pub plan: OutputPlan,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("fs error: {0}")]
    Fs(#[from] crate::fsutil::FsError),

    #[error("backend conflict at {path}: {message}")]
    Conflict { path: String, message: String },

    #[error("backend unsupported: {message}")]
    Unsupported { message: String },
}

pub trait Backend {
    fn prepare(&self, repo_root: &Path, plan: &OutputPlan) -> Result<BackendSession, BackendError>;

    fn apply(
        &self,
        session: &mut BackendSession,
        outputs: &[RenderedOutput],
    ) -> Result<ApplyReport, BackendError>;
}
