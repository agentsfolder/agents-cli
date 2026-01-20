use crate::stamps::DriftStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Create,
    Update,
    Delete,
    Noop,

    /// Planned output path already exists but is not managed by agents.
    UnmanagedExists,

    /// Managed output exists but differs from planned due to manual edits.
    Drifted,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub path: String,
    pub kind: DiffKind,

    /// Drift status for this path, when applicable.
    pub drift: Option<DriftStatus>,

    /// Optional human-readable details for diagnostics.
    pub details: Option<String>,

    /// Optional unified diff (without stamps).
    pub unified_diff: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DiffReport {
    pub entries: Vec<DiffEntry>,
}
