use std::path::Path;

use agents_core::cleanup;
use agents_core::fsutil;
use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::model::BackendKind;
use agents_core::outputs::{plan_outputs, PlanError};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::{driftx, driftx::DiffKind};
use std::collections::BTreeSet;

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
    report.items.extend(drift_check(&ctx));
    report.items.extend(prereqs_check(&ctx));
    report.items.extend(state_file_check(&ctx));

    if opts.fix {
        report.items.extend(apply_fixes(&ctx));
    }
    report.normalize_order();

    for item in &report.items {
        println!("{}: {}: {}", item.level, item.check, item.message);
        for c in &item.context {
            println!("  {c}");
        }
    }

    println!(
        "doctor: errors={} warnings={} ci={} fix={}",
        report
            .items
            .iter()
            .filter(|i| i.level == DoctorLevel::Error)
            .count(),
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

fn apply_fixes(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let mut items = vec![];

    // Fix: ensure `.agents/state/.gitignore` exists and contains `state.yaml`.
    match fix_state_gitignore(&ctx.repo_root) {
        Ok(changed) => {
            if changed {
                items.push(DoctorItem {
                    level: DoctorLevel::Info,
                    check: "fix".to_string(),
                    message: "updated .agents/state/.gitignore".to_string(),
                    context: vec!["path: .agents/state/.gitignore".to_string()],
                });
            }
        }
        Err(e) => items.push(DoctorItem {
            level: DoctorLevel::Error,
            check: "fix".to_string(),
            message: "failed to update .agents/state/.gitignore".to_string(),
            context: vec![e.to_string()],
        }),
    }

    // Fix (optional): remove stale generated outputs that are no longer planned.
    if let (Some(repo), Some(effective)) = (&ctx.repo, &ctx.effective) {
        let mut agent_ids = repo.manifest.enabled.adapters.clone();
        agent_ids.sort();

        let mut deleted_paths: Vec<fsutil::RepoPath> = vec![];
        for agent_id in &agent_ids {
            let plan = match plan_outputs(&ctx.repo_root, repo.clone(), effective, agent_id) {
                Ok(p) => p.plan,
                Err(_) => continue,
            };

            let planned: BTreeSet<String> = plan
                .outputs
                .iter()
                .map(|o| o.path.as_str().to_string())
                .collect();
            let stale = match driftx::detect_stale_generated(&ctx.repo_root, agent_id, &planned) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for e in stale {
                let rp = match fsutil::repo_relpath_noexist(
                    &ctx.repo_root,
                    std::path::Path::new(&e.path),
                ) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                deleted_paths.push(rp);
            }
        }

        deleted_paths.sort();
        deleted_paths.dedup();
        if !deleted_paths.is_empty() {
            match cleanup::delete_paths(&ctx.repo_root, &deleted_paths, false) {
                Ok(rep) => {
                    for p in rep.deleted {
                        items.push(DoctorItem {
                            level: DoctorLevel::Info,
                            check: "fix".to_string(),
                            message: "deleted stale generated file".to_string(),
                            context: vec![format!("path: {}", p.as_str())],
                        });
                    }
                }
                Err(e) => items.push(DoctorItem {
                    level: DoctorLevel::Error,
                    check: "fix".to_string(),
                    message: "failed to delete stale generated files".to_string(),
                    context: vec![e.to_string()],
                }),
            }
        }
    }

    items
}

fn fix_state_gitignore(repo_root: &Path) -> Result<bool, fsutil::FsError> {
    let dir = repo_root.join(".agents/state");
    std::fs::create_dir_all(&dir).map_err(|e| fsutil::FsError::Io {
        path: dir.clone(),
        source: e,
    })?;

    let p = dir.join(".gitignore");
    let mut changed = false;
    let mut content = if p.is_file() {
        fsutil::read_to_string(&p)?
    } else {
        changed = true;
        String::new()
    };

    for rule in ["state.yaml", "explain/"] {
        let rooted = format!("/{rule}");
        let has = content.lines().any(|l| {
            let t = l.trim();
            t == rule || t == rooted
        });
        if !has {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(rule);
            content.push('\n');
            changed = true;
        }
    }

    if changed {
        fsutil::atomic_write(&p, content.as_bytes())?;
    }

    Ok(changed)
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
                format!(
                    "path: {}",
                    fsutil::display_repo_path(&ctx.repo_root, &err.path)
                        .unwrap_or_else(|_| err.path.display().to_string())
                ),
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
                PlanError::PathCollision { .. }
                | PlanError::SurfaceCollision { .. }
                | PlanError::SharedOwnerViolation { .. } => (
                    if ctx.ci {
                        DoctorLevel::Error
                    } else {
                        DoctorLevel::Warning
                    },
                    "collision detected",
                ),
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

fn drift_check(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let Some(repo) = &ctx.repo else {
        return vec![];
    };
    let Some(effective) = &ctx.effective else {
        return vec![];
    };

    let mut agent_ids = repo.manifest.enabled.adapters.clone();
    agent_ids.sort();

    let mut drifted: Vec<String> = vec![];
    let mut unmanaged: Vec<String> = vec![];
    let mut stale: Vec<String> = vec![];
    let mut errors: Vec<String> = vec![];

    for agent_id in &agent_ids {
        let plan = match plan_outputs(&ctx.repo_root, repo.clone(), effective, agent_id) {
            Ok(p) => p.plan,
            Err(e) => {
                errors.push(format!("{agent_id}: {e}"));
                continue;
            }
        };

        let report = match driftx::diff_plan(&ctx.repo_root, &plan) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{agent_id}: {e}"));
                continue;
            }
        };

        for entry in report.entries {
            match entry.kind {
                DiffKind::Drifted => drifted.push(format!("{agent_id}:{}", entry.path)),
                DiffKind::UnmanagedExists => unmanaged.push(format!("{agent_id}:{}", entry.path)),
                DiffKind::Delete => stale.push(format!("{agent_id}:{}", entry.path)),
                _ => {}
            }
        }
    }

    drifted.sort();
    unmanaged.sort();
    stale.sort();
    errors.sort();

    let mut items: Vec<DoctorItem> = vec![];
    if !errors.is_empty() {
        items.push(DoctorItem {
            level: DoctorLevel::Error,
            check: "drift".to_string(),
            message: "drift check failed".to_string(),
            context: errors,
        });
        return items;
    }

    if !unmanaged.is_empty() {
        items.push(DoctorItem {
            level: if ctx.ci {
                DoctorLevel::Error
            } else {
                DoctorLevel::Warning
            },
            check: "drift".to_string(),
            message: "unmanaged files block sync".to_string(),
            context: unmanaged,
        });
    }

    if !drifted.is_empty() {
        items.push(DoctorItem {
            level: if ctx.ci {
                DoctorLevel::Error
            } else {
                DoctorLevel::Warning
            },
            check: "drift".to_string(),
            message: "generated files drifted".to_string(),
            context: drifted,
        });
    }

    if !stale.is_empty() {
        items.push(DoctorItem {
            level: DoctorLevel::Warning,
            check: "drift".to_string(),
            message: "stale generated files can be removed".to_string(),
            context: stale,
        });
    }

    if items.is_empty() {
        items.push(DoctorItem {
            level: DoctorLevel::Info,
            check: "drift".to_string(),
            message: "no drift detected".to_string(),
            context: vec![],
        });
    }

    items
}

fn prereqs_check(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let Some(repo) = &ctx.repo else {
        return vec![];
    };
    let Some(effective) = &ctx.effective else {
        return vec![];
    };

    let mut needs_docker = false;

    // Current effective backend.
    if effective.backend == BackendKind::VfsContainer {
        needs_docker = true;
    }

    // Manifest defaults/byAgent.
    if repo.manifest.defaults.backend == Some(BackendKind::VfsContainer) {
        needs_docker = true;
    }
    if let Some(backends) = &repo.manifest.backends {
        if backends.default == Some(BackendKind::VfsContainer) {
            needs_docker = true;
        }
        if backends
            .by_agent
            .values()
            .any(|b| *b == BackendKind::VfsContainer)
        {
            needs_docker = true;
        }
    }

    // Adapter defaults.
    if repo
        .adapters
        .values()
        .any(|a| a.backend_defaults.preferred == BackendKind::VfsContainer)
    {
        needs_docker = true;
    }

    if !needs_docker {
        return vec![DoctorItem {
            level: DoctorLevel::Info,
            check: "prereqs".to_string(),
            message: "docker not required".to_string(),
            context: vec![],
        }];
    }

    let ok = std::process::Command::new("docker")
        .arg("--version")
        .output()
        .is_ok();

    if ok {
        vec![DoctorItem {
            level: DoctorLevel::Info,
            check: "prereqs".to_string(),
            message: "docker available".to_string(),
            context: vec![],
        }]
    } else {
        vec![DoctorItem {
            level: if ctx.ci {
                DoctorLevel::Error
            } else {
                DoctorLevel::Warning
            },
            check: "prereqs".to_string(),
            message: "docker is required for vfs_container backend".to_string(),
            context: vec![
                "hint: install Docker Desktop (or docker CLI)".to_string(),
                "hint: run `docker --version`".to_string(),
            ],
        }]
    }
}

fn state_file_check(ctx: &DoctorContext) -> Vec<DoctorItem> {
    let p = ctx.repo_root.join(".agents/state/.gitignore");
    if !p.is_file() {
        return vec![DoctorItem {
            level: DoctorLevel::Warning,
            check: "state".to_string(),
            message: "missing .agents/state/.gitignore".to_string(),
            context: vec![
                format!("path: {}", p.display()),
                "hint: run `agents doctor --fix`".to_string(),
            ],
        }];
    }

    let content = match fsutil::read_to_string(&p) {
        Ok(s) => s,
        Err(e) => {
            return vec![DoctorItem {
                level: DoctorLevel::Error,
                check: "state".to_string(),
                message: "failed to read .agents/state/.gitignore".to_string(),
                context: vec![e.to_string()],
            }]
        }
    };

    let has_state_yaml = content
        .lines()
        .any(|l| l.trim() == "state.yaml" || l.trim() == "/state.yaml");
    let has_explain = content
        .lines()
        .any(|l| l.trim() == "explain/" || l.trim() == "/explain/");

    if !has_state_yaml {
        return vec![DoctorItem {
            level: DoctorLevel::Warning,
            check: "state".to_string(),
            message: "state.yaml is not ignored".to_string(),
            context: vec![
                format!("path: {}", p.display()),
                "hint: add `state.yaml` to .agents/state/.gitignore".to_string(),
                "hint: or run `agents doctor --fix`".to_string(),
            ],
        }];
    }

    if !has_explain {
        return vec![DoctorItem {
            level: DoctorLevel::Warning,
            check: "state".to_string(),
            message: "explain/ is not ignored".to_string(),
            context: vec![
                format!("path: {}", p.display()),
                "hint: add `explain/` to .agents/state/.gitignore".to_string(),
                "hint: or run `agents doctor --fix`".to_string(),
            ],
        }];
    }

    vec![DoctorItem {
        level: DoctorLevel::Info,
        check: "state".to_string(),
        message: "state gitignore ok".to_string(),
        context: vec![],
    }]
}
