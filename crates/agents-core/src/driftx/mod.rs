mod compare;
mod diff;
mod types;

pub use compare::{diff_plan, DriftxError};
pub use diff::unified_diff_for;
pub use types::{DiffEntry, DiffKind, DiffReport};
