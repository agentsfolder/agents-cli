use std::path::Path;

use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions, RepoConfig};
use agents_core::matwiz::{Backend, MaterializeBackend};
use agents_core::model::BackendKind;
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::stamps::{classify, parse_stamp};

use crate::{AppError, ErrorCategory};

#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub agent: String,
    pub backend: Option<BackendKind>,
    pub verbose: bool,
}

pub fn cmd_sync(repo_root: &Path, opts: SyncOptions) -> Result<(), AppError> {
    // Load repo config.
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

    let selected_backend = select_backend(&repo, &opts.agent, opts.backend);

    // Resolve effective config.
    let resolver = Resolver::new(repo.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo_root.to_path_buf();
    req.override_backend = Some(selected_backend);

    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    // Plan outputs.
    let plan_res =
        plan_outputs(repo_root, repo.clone(), &effective, &opts.agent).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![],
        })?;

    match selected_backend {
        BackendKind::Materialize => {
            let backend = MaterializeBackend;
            let mut session = backend.prepare(repo_root, &plan_res.plan).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;

            let mut rendered_outputs: Vec<agents_core::matwiz::RenderedOutput> = vec![];
            for out in &plan_res.plan.outputs {
                let rendered = render_planned_output(repo_root, out).map_err(|e| AppError {
                    category: ErrorCategory::Io,
                    message: e.to_string(),
                    context: vec![format!("path: {}", out.path.as_str())],
                })?;

                let stamp = parse_stamp(&rendered.content_with_stamp).ok_or_else(|| AppError {
                    category: ErrorCategory::Io,
                    message: "rendered output missing stamp".to_string(),
                    context: vec![format!("path: {}", out.path.as_str())],
                })?;

                let drift_status = classify(
                    &repo_root.join(out.path.as_str()),
                    &rendered.content_without_stamp,
                    &out.drift_detection,
                )
                .map_err(|e| AppError {
                    category: ErrorCategory::Io,
                    message: e.to_string(),
                    context: vec![format!("path: {}", out.path.as_str())],
                })?;

                rendered_outputs.push(agents_core::matwiz::RenderedOutput {
                    path: out.path.clone(),
                    bytes: rendered.content_with_stamp.into_bytes(),
                    stamp_meta: stamp.meta,
                    drift_status,
                });
            }

            let report = backend.apply(&mut session, &rendered_outputs).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;

            if !report.conflicts.is_empty() {
                let msg = report
                    .conflict_details
                    .first()
                    .map(|c| c.message.clone())
                    .unwrap_or_else(|| "conflicts detected".to_string());
                return Err(AppError {
                    category: ErrorCategory::Conflict,
                    message: msg,
                    context: report
                        .conflict_details
                        .first()
                        .map(|c| c.hints.clone())
                        .unwrap_or_default(),
                });
            }

            if opts.verbose {
                for p in &report.written {
                    println!("write: {}", p.as_str());
                }
                for p in &report.skipped {
                    println!("skip: {}", p.as_str());
                }
            }

            println!(
                "sync: written={} skipped={} conflict={}",
                report.written.len(),
                report.skipped.len(),
                report.conflicts.len()
            );

            Ok(())
        }
        BackendKind::VfsContainer | BackendKind::VfsMount => Err(AppError {
            category: ErrorCategory::ExternalToolMissing,
            message: format!("backend not implemented yet: {selected_backend:?}"),
            context: vec![],
        }),
    }
}

fn select_backend(repo: &RepoConfig, agent: &str, cli: Option<BackendKind>) -> BackendKind {
    if let Some(b) = cli {
        return b;
    }

    if let Some(backends) = &repo.manifest.backends {
        if let Some(b) = backends.by_agent.get(agent) {
            return *b;
        }
        if let Some(b) = backends.default {
            return b;
        }
    }

    if let Some(b) = repo.manifest.defaults.backend {
        return b;
    }

    if let Some(adapter) = repo.adapters.get(agent) {
        return adapter.backend_defaults.preferred;
    }

    BackendKind::VfsContainer
}
