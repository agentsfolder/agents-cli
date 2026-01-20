mod compare;
mod diff;
mod stale;
mod types;

pub use compare::{diff_plan, DriftxError};
pub use diff::unified_diff_for;
pub use stale::detect_stale_generated;
pub use types::{DiffEntry, DiffKind, DiffReport};
