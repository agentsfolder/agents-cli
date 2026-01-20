use std::path::Path;

use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::fsutil;
use agents_core::outputs::{plan_outputs, PlanError};
use agents_core::resolv::{ResolutionRequest, Resolver};

use crate::{AppError, ErrorCategory};

use super::{DoctorContext, DoctorItem, DoctorLevel, DoctorReport};

#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub fix: bool,
    pub ci: bool,
}

pub fn cmd_doctor(repo_root: &Path, opts: DoctorOptions) -> Result<(), AppError> {
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

    let resolver = Resolver::new(repo.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo_root.to_path_buf();
    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let ctx = DoctorContext {
        repo_root: repo_root.to_path_buf(),
        repo: Some(repo),
        effective: Some(effective),
        ci: opts.ci,
        fix: opts.fix,
    };

    let mut report = DoctorReport::default();
    report.items.extend(schema_check(&ctx));
    report.items.extend(collision_check(&ctx));
    report.normalize_order();

    for item in &report.items {
        println!("{}: {}: {}", item.level, item.check, item.message);
        for c in &item.context {
            println!("  {c}");
        }
    }

    println!(
        "doctor: errors={} warnings={} ci={} fix={}",
        report.items.iter().filter(|i| i.level == DoctorLevel::Error).count(),
        report
            .items
            .iter()
            .filter(|i| i.level == DoctorLevel::Warning)
            .count(),
        opts.ci,
        opts.fix
    );

    let fail = report.has_errors() || (opts.ci && report.has_warnings());
    if fail {
        return Err(AppError {
            category: ErrorCategory::SchemaInvalid,
            message: "doctor found issues".to_string(),
            context: vec!["hint: run `agents diff` or `agents validate`".to_string()],
        });
    }

    Ok(())
}

fn schema_check(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let schemas_dir = ctx.repo_root.join(".agents/schemas");
    if !schemas_dir.is_dir() {
        let level = if ctx.ci {
            DoctorLevel::Error
        } else {
            DoctorLevel::Warning
        };
        return vec![DoctorItem {
            level,
            check: "schemas".to_string(),
            message: "schemas directory missing".to_string(),
            context: vec![format!("path: {}", schemas_dir.display())],
        }];
    }

    match agents_core::schemas::validate_repo(&ctx.repo_root) {
        Ok(()) => vec![DoctorItem {
            level: DoctorLevel::Info,
            check: "schemas".to_string(),
            message: "schemas valid".to_string(),
            context: vec![],
        }],
        Err(err) => vec![DoctorItem {
            level: DoctorLevel::Error,
            check: "schemas".to_string(),
            message: "schema invalid".to_string(),
            context: vec![
                format!("path: {}", fsutil::display_repo_path(&ctx.repo_root, &err.path).unwrap_or_else(|_| err.path.display().to_string())),
                format!("schema: {}", err.schema),
                format!("pointer: {}", err.pointer),
                err.message,
            ],
        }],
    }
}

fn collision_check(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let Some(repo) = &ctx.repo else {
        return vec![];
    };
    let Some(effective) = &ctx.effective else {
        return vec![];
    };

    let mut agent_ids = repo.manifest.enabled.adapters.clone();
    agent_ids.sort();

    let mut items: Vec<DoctorItem> = vec![];
    for agent_id in agent_ids {
        let res = plan_outputs(&ctx.repo_root, repo.clone(), effective, &agent_id);
        if let Err(err) = res {
            let (level, msg) = match &err {
                PlanError::PathCollision { .. } | PlanError::SurfaceCollision { .. } => {
                    (if ctx.ci { DoctorLevel::Error } else { DoctorLevel::Warning }, "collision detected")
                }
                _ => (DoctorLevel::Error, "output planning failed"),
            };

            items.push(DoctorItem {
                level,
                check: "collisions".to_string(),
                message: format!("{msg} for adapter {agent_id}"),
                context: vec![err.to_string()],
            });
        }
    }

    if items.is_empty() {
        items.push(DoctorItem {
            level: DoctorLevel::Info,
            check: "collisions".to_string(),
            message: "no collisions detected".to_string(),
            context: vec![],
        });
    }

    items
}
