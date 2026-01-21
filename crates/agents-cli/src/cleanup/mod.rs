use std::path::Path;

use agents_core::cleanup::{delete_paths, identify_deletable};
use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::model::ConfirmationType;
use agents_core::resolv::{ResolutionRequest, Resolver};

use crate::{AppError, ErrorCategory};

#[derive(Debug, Clone)]
pub struct CleanOptions {
    pub agent: Option<String>,
    pub dry_run: bool,
    pub yes: bool,
}

pub fn cmd_clean(repo_root: &Path, opts: CleanOptions) -> Result<(), AppError> {
    let (repo, _report) = load_repo_config(
        repo_root,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .map_err(|e| match e {
        LoadError::NotInitialized { .. } => AppError::not_initialized(repo_root),
        other => AppError {
            category: ErrorCategory::Io,
            message: other.to_string(),
            context: vec![],
        },
    })?;

    // Validate schemas best-effort.
    let _ = agents_core::schemas::validate_repo(repo_root);

    // Resolve effective config.
    let resolver = Resolver::new(repo.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo_root.to_path_buf();
    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    // Safety: require confirmation if policy requests it.
    if !opts.dry_run {
        let policy = repo
            .policies
            .get(&effective.policy_id)
            .ok_or_else(|| AppError {
                category: ErrorCategory::Io,
                message: format!("missing policy {}", effective.policy_id),
                context: vec![],
            })?;

        let requires_confirm = policy
            .confirmations
            .required_for
            .iter()
            .any(|c| *c == ConfirmationType::Delete);

        if requires_confirm && !opts.yes {
            return Err(AppError {
                category: ErrorCategory::PolicyDenied,
                message: "delete requires confirmation by policy".to_string(),
                context: vec![
                    "hint: rerun with `--yes` to confirm delete".to_string(),
                    "hint: or use `--dry-run` to preview deletions".to_string(),
                ],
            });
        }
    }

    let mut agent_ids: Vec<String> = if let Some(a) = &opts.agent {
        vec![a.clone()]
    } else {
        repo.adapters.keys().cloned().collect()
    };
    agent_ids.sort();

    let mut identify =
        identify_deletable(repo_root, &repo, &effective, &agent_ids).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![],
        })?;

    identify
        .skipped
        .sort_by(|a, b| a.path.as_str().cmp(b.path.as_str()));

    let delete =
        delete_paths(repo_root, &identify.eligible, opts.dry_run).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![],
        })?;

    let verb = if opts.dry_run {
        "would-delete"
    } else {
        "delete"
    };
    for p in &delete.deleted {
        println!("{verb}: {}", p.as_str());
    }

    for s in &identify.skipped {
        let reason = match s.reason {
            agents_core::cleanup::SkipReason::NoStamp => "no_stamp",
            agents_core::cleanup::SkipReason::NotGeneratedByAgents => "not_generated",
            agents_core::cleanup::SkipReason::DifferentAdapter => "different_adapter",
            agents_core::cleanup::SkipReason::Drifted => "drifted",
        };
        println!("skip: {} ({reason})", s.path.as_str());
    }

    if !opts.dry_run {
        for d in &delete.pruned_dirs {
            println!("prune: {}", d.as_str());
        }
    }

    println!(
        "clean: deleted={} skipped={} dry_run={}",
        delete.deleted.len(),
        identify.skipped.len(),
        opts.dry_run
    );

    Ok(())
}
