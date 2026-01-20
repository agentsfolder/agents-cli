use std::collections::BTreeMap;
use std::path::Path;

use crate::fsutil::{self, RepoPath};
use crate::loadag::RepoConfig;
use crate::outputs::plan_outputs;
use crate::resolv::EffectiveConfig;
use crate::stamps::{compute_sha256_hex, parse_stamp, strip_existing_stamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    NoStamp,
    NotGeneratedByAgents,
    DifferentAdapter,
    Drifted,
}

#[derive(Debug, Clone)]
pub struct SkippedPath {
    pub path: RepoPath,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, Default)]
pub struct IdentifyReport {
    pub eligible: Vec<RepoPath>,
    pub skipped: Vec<SkippedPath>,
}

#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("outputs plan error: {0}")]
    Plan(#[from] crate::outputs::PlanError),

    #[error("fs error: {0}")]
    Fs(#[from] fsutil::FsError),
}

/// Identify generated files that are safe to delete.
///
/// Safety rule (v1): eligible if and only if:
/// - file exists
/// - a valid stamp is present
/// - stamp generator is `agents`
/// - stamp adapter matches the requested agent
/// - the current content (without stamp) matches the stamped sha256
pub fn identify_deletable(
    repo_root: &Path,
    repo: &RepoConfig,
    effective: &EffectiveConfig,
    agent_ids: &[String],
) -> Result<IdentifyReport, CleanupError> {
    let mut eligible_by_path: BTreeMap<String, RepoPath> = BTreeMap::new();
    let mut skipped: Vec<SkippedPath> = vec![];

    for agent_id in agent_ids {
        let plan_res = plan_outputs(repo_root, repo.clone(), effective, agent_id)?;
        for out in &plan_res.plan.outputs {
            let abs = repo_root.join(out.path.as_str());
            if !abs.is_file() {
                continue;
            }

            let existing = fsutil::read_to_string(&abs)?;
            let Some(stamp) = parse_stamp(&existing) else {
                skipped.push(SkippedPath {
                    path: out.path.clone(),
                    reason: SkipReason::NoStamp,
                });
                continue;
            };

            if stamp.meta.generator != "agents" {
                skipped.push(SkippedPath {
                    path: out.path.clone(),
                    reason: SkipReason::NotGeneratedByAgents,
                });
                continue;
            }

            if stamp.meta.adapter_agent_id != *agent_id {
                skipped.push(SkippedPath {
                    path: out.path.clone(),
                    reason: SkipReason::DifferentAdapter,
                });
                continue;
            }

            let (without_stamp, _found) = strip_existing_stamp(&existing);
            let current_hash = compute_sha256_hex(&without_stamp);
            if current_hash != stamp.meta.content_sha256 {
                skipped.push(SkippedPath {
                    path: out.path.clone(),
                    reason: SkipReason::Drifted,
                });
                continue;
            }

            eligible_by_path
                .entry(out.path.as_str().to_string())
                .or_insert_with(|| out.path.clone());
        }
    }

    let eligible: Vec<RepoPath> = eligible_by_path.into_values().collect();
    Ok(IdentifyReport { eligible, skipped })
}
