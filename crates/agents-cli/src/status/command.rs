use std::path::Path;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::skillpl::SkillPlanner;

use crate::status::StatusReport;
use crate::{AppError, ErrorCategory, OutputMode};

pub fn cmd_status(repo_root: &Path, output: OutputMode) -> Result<(), AppError> {
    let (repo, report) = load_repo_config(
        repo_root,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    // Validate schemas (best-effort: only if schemas exist).
    let _ = agents_core::schemas::validate_repo(repo_root);

    let resolver = Resolver::new(repo.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo_root.to_path_buf();

    // If state.yaml exists, resolution will likely use it. Expose this as a hint.
    let state_influences = repo.state.is_some();

    // If user overlay is enabled, mention it. (The actual overlay root resolution is handled in agents-core.)
    let user_overlay_enabled = repo
        .manifest
        .resolution
        .as_ref()
        .map(|r| r.enable_user_overlay)
        .unwrap_or(true);

    if user_overlay_enabled {
        req.enable_user_overlay = true;
    }

    // If the repo warns about missing schemas, we keep it as a hint as well.
    let has_load_warnings = !report.warnings.is_empty();

    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let planner = SkillPlanner::new(repo);
    let skills = planner.plan(&effective, None).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let mut report = StatusReport {
        repo_root: repo_root.display().to_string(),
        effective_mode: effective.mode_id,
        effective_policy: effective.policy_id,
        effective_profile: effective.profile,
        effective_backend: effective.backend,
        scopes_matched: effective.scopes_matched.into_iter().map(|s| s.id).collect(),
        skills_enabled: skills.enabled.into_iter().map(|s| s.id).collect(),
        agent_id: None,
        hints: vec![],
    };

    if state_influences {
        report.hints.push(
            "hint: .agents/state/state.yaml is present and may influence mode/profile/backend"
                .to_string(),
        );
    }

    if user_overlay_enabled {
        report
            .hints
            .push("hint: user overlay is enabled (disable via manifest.resolution.enableUserOverlay=false)".to_string());
    }

    if has_load_warnings {
        report.hints.push(
            "hint: repo has .agents warnings (run `agents validate` for details)".to_string(),
        );
    }

    if has_load_warnings {
        report.hints.push(
            "hint: repo has .agents warnings (run `agents validate` for details)".to_string(),
        );
    }

    match output {
        OutputMode::Human => {
            print!("{}", report.render_human());
            Ok(())
        }
        OutputMode::Json => Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: "--json output is not implemented yet".to_string(),
            context: vec![],
        }),
    }
}
