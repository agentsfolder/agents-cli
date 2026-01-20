use std::path::Path;

use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions, RepoConfig};
use agents_core::model::{BackendKind, WriteMode};
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::stamps::parse_stamp;

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
            // v1: materialize directly to repo.
            for out in &plan_res.plan.outputs {
                let rendered = render_planned_output(repo_root, out).map_err(|e| AppError {
                    category: ErrorCategory::Io,
                    message: e.to_string(),
                    context: vec![format!("path: {}", out.path.as_str())],
                })?;

                let dest = repo_root.join(out.path.as_str());

                // v1: honor writePolicy modes.
                let mode = out.write_policy.mode.unwrap_or(WriteMode::IfGenerated);
                if mode == WriteMode::Never {
                    if opts.verbose {
                        println!("skip: {} (writePolicy=never)", out.path.as_str());
                    }
                    continue;
                }

                if mode == WriteMode::IfGenerated && dest.exists() {
                    // Refuse to overwrite unmanaged files.
                    let existing =
                        agents_core::fsutil::read_to_string(&dest).map_err(|e| AppError {
                            category: ErrorCategory::Io,
                            message: e.to_string(),
                            context: vec![],
                        })?;

                    if parse_stamp(&existing).is_none() {
                        return Err(AppError {
                            category: ErrorCategory::Conflict,
                            message: format!("unmanaged file exists at {}", out.path.as_str()),
                            context: vec![
                                "hint: run `agents diff --agent <id>` to see conflicts".to_string(),
                                "hint: change output.writePolicy.mode to `always` to force overwrite".to_string(),
                            ],
                        });
                    }
                }

                agents_core::fsutil::atomic_write(&dest, rendered.content_with_stamp.as_bytes())
                    .map_err(|e| AppError {
                        category: ErrorCategory::Io,
                        message: e.to_string(),
                        context: vec![],
                    })?;

                if opts.verbose {
                    println!("write: {}", out.path.as_str());
                }
            }

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
