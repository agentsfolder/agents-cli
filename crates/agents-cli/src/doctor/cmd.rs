use std::path::Path;

use agents_core::fsutil;

use crate::{AppError, ErrorCategory};

use super::{DoctorContext, DoctorItem, DoctorLevel, DoctorReport};

#[derive(Debug, Clone)]
pub struct DoctorOptions {
    pub fix: bool,
    pub ci: bool,
}

pub fn cmd_doctor(repo_root: &Path, opts: DoctorOptions) -> Result<(), AppError> {
    let ctx = DoctorContext {
        repo_root: repo_root.to_path_buf(),
        repo: None,
        effective: None,
        ci: opts.ci,
        fix: opts.fix,
    };

    let mut report = DoctorReport::default();
    report.items.extend(schema_check(&ctx));
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
