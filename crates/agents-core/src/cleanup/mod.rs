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

#[derive(Debug, Clone, Default)]
pub struct DeleteReport {
    pub deleted: Vec<RepoPath>,
    pub pruned_dirs: Vec<RepoPath>,
}

#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("outputs plan error: {0}")]
    Plan(#[from] crate::outputs::PlanError),

    #[error("fs error: {0}")]
    Fs(#[from] fsutil::FsError),
}

pub fn delete_paths(
    repo_root: &Path,
    paths: &[RepoPath],
    dry_run: bool,
) -> Result<DeleteReport, CleanupError> {
    let mut report = DeleteReport::default();

    for rp in paths {
        let abs = repo_root.join(rp.as_str());
        if !abs.exists() {
            continue;
        }

        // Safety: ensure we are deleting inside the repo root.
        let _ = fsutil::repo_relpath(repo_root, &abs)?;

        if !dry_run {
            std::fs::remove_file(&abs).map_err(|e| fsutil::FsError::Io {
                path: abs.clone(),
                source: e,
            })?;
        }

        report.deleted.push(rp.clone());

        if !dry_run {
            prune_empty_parents(repo_root, &abs, &mut report.pruned_dirs)?;
        }
    }

    Ok(report)
}

fn prune_empty_parents(
    repo_root: &Path,
    deleted_file: &Path,
    pruned: &mut Vec<RepoPath>,
) -> Result<(), CleanupError> {
    let root_abs = repo_root.canonicalize().map_err(|e| fsutil::FsError::Io {
        path: repo_root.to_path_buf(),
        source: e,
    })?;

    let mut cur = deleted_file.parent().map(Path::to_path_buf);
    while let Some(dir) = cur {
        let dir_abs = dir.canonicalize().map_err(|e| fsutil::FsError::Io {
            path: dir.clone(),
            source: e,
        })?;

        if dir_abs == root_abs {
            break;
        }

        let mut it = std::fs::read_dir(&dir).map_err(|e| fsutil::FsError::Io {
            path: dir.clone(),
            source: e,
        })?;
        let is_empty = it.next().is_none();
        if !is_empty {
            break;
        }

        std::fs::remove_dir(&dir).map_err(|e| fsutil::FsError::Io {
            path: dir.clone(),
            source: e,
        })?;

        if let Ok(rp) = fsutil::repo_relpath(&root_abs, &dir_abs) {
            pruned.push(rp);
        }

        cur = dir.parent().map(Path::to_path_buf);
    }

    Ok(())
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
