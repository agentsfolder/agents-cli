mod compare;
mod types;

pub use compare::{diff_plan, DriftxError};
pub use types::{DiffEntry, DiffKind, DiffReport};
