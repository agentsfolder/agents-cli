use std::path::{Path, PathBuf};

use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};

use crate::{AppError, ErrorCategory};

#[derive(Debug, Clone)]
pub struct PreviewOptions {
    pub agent: String,
    pub backend: Option<agents_core::model::BackendKind>,
    pub mode: Option<String>,
    pub profile: Option<String>,
    pub keep_temp: bool,
}

pub fn cmd_preview(repo_root: &Path, opts: PreviewOptions) -> Result<(), AppError> {
    // Load repo config
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
    req.override_mode = opts.mode.clone();
    req.override_profile = opts.profile.clone();
    if let Some(b) = opts.backend {
        req.override_backend = Some(b);
    }

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

    let tmp = agents_core::fsutil::temp_generation_dir("agents-preview").map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let tmp_path = tmp.path().to_path_buf();

    // Render all planned outputs into temp dir.
    for out in &plan_res.plan.outputs {
        let rendered = render_planned_output(repo_root, out).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", out.path.as_str())],
        })?;

        let dest: PathBuf = tmp_path.join(out.path.as_str());
        agents_core::fsutil::atomic_write(&dest, rendered.content_with_stamp.as_bytes()).map_err(
            |e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            },
        )?;

        println!("preview: {} -> {}", out.path.as_str(), dest.display());
    }

    if opts.keep_temp {
        println!("temp: {}", tmp_path.display());
        std::mem::forget(tmp);
    }

    Ok(())
}
