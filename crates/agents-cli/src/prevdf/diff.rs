use std::path::Path;

use agents_core::driftx::{diff_plan, DiffKind};
use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};

use crate::{AppError, ErrorCategory};

#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub agent: String,
    pub show: bool,
}

pub fn cmd_diff(repo_root: &Path, opts: DiffOptions) -> Result<(), AppError> {
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

    let resolver = Resolver::new(repo.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo_root.to_path_buf();

    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let plan_res =
        plan_outputs(repo_root, repo.clone(), &effective, &opts.agent).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![],
        })?;

    // Ensure planned content can be rendered now (forces renderer errors early).
    for out in &plan_res.plan.outputs {
        let _ = render_planned_output(repo_root, out).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", out.path.as_str())],
        })?;
    }

    let report = diff_plan(repo_root, &plan_res.plan).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let mut creates = 0usize;
    let mut updates = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;
    let mut deletes = 0usize;

    for e in &report.entries {
        match e.kind {
            DiffKind::Create => creates += 1,
            DiffKind::Update => updates += 1,
            DiffKind::Noop => noops += 1,
            DiffKind::UnmanagedExists | DiffKind::Drifted => conflicts += 1,
            DiffKind::Delete => deletes += 1,
        }
    }

    println!(
        "changes: create={} update={} delete={} noop={} conflict={}",
        creates, updates, deletes, noops, conflicts
    );

    for e in &report.entries {
        let label = match e.kind {
            DiffKind::Create => "CREATE",
            DiffKind::Update => "UPDATE",
            DiffKind::Delete => "DELETE",
            DiffKind::Noop => "NOOP",
            DiffKind::UnmanagedExists => "CONFLICT(unmanaged)",
            DiffKind::Drifted => "CONFLICT(drifted)",
        };

        println!("{label}: {}", e.path);

        if opts.show {
            if let Some(d) = &e.unified_diff {
                print!("{d}");
            }
        }
    }

    Ok(())
}
