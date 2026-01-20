use std::path::Path;

use crate::fsutil;
use crate::model::{DriftDetection, DriftMethod};

use super::{compute_sha256_hex, parse_stamp, strip_existing_stamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftStatus {
    Missing,
    Unmanaged,
    Clean,
    Drifted,
}

pub fn classify(
    path: &Path,
    planned_content_without_stamp: &str,
    drift: &DriftDetection,
) -> fsutil::FsResult<DriftStatus> {
    let method = drift.method.unwrap_or(DriftMethod::Sha256);

    if !path.exists() {
        return Ok(DriftStatus::Missing);
    }

    match method {
        DriftMethod::None => {
            // For v1, if drift detection is disabled we still distinguish managed/unmanaged.
            let existing = fsutil::read_to_string(path)?;
            if parse_stamp(&existing).is_some() {
                Ok(DriftStatus::Clean)
            } else {
                Ok(DriftStatus::Unmanaged)
            }
        }
        DriftMethod::MtimeOnly | DriftMethod::Sha256 => {
            let existing = fsutil::read_to_string(path)?;
            let (existing_without_stamp, stamp) = strip_existing_stamp(&existing);

            let Some(_stamp) = stamp else {
                return Ok(DriftStatus::Unmanaged);
            };

            // Compare using sha256 of canonical (newline-normalized) content without stamp.
            //
            // Note: for `mtime_only`, this is a temporary behavior until we have a baseline mtime
            // recorded (planned vs last-written). Treating it as sha256 keeps semantics safe.
            let planned_hash = compute_sha256_hex(planned_content_without_stamp);
            let existing_hash = compute_sha256_hex(&existing_without_stamp);

            if existing_hash == planned_hash {
                Ok(DriftStatus::Clean)
            } else {
                Ok(DriftStatus::Drifted)
            }
        }
    }
}
