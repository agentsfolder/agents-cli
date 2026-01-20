use std::collections::BTreeSet;
use std::path::Path;

use crate::fsutil;
use crate::outputs::{OutputPlan, PlannedOutput};
use crate::stamps::{classify, strip_existing_stamp, DriftStatus};
use crate::templ::TemplateEngine;

use super::{detect_stale_generated, unified_diff_for, DiffEntry, DiffKind, DiffReport};

#[derive(Debug, thiserror::Error)]
pub enum DriftxError {
    #[error("template error: {0}")]
    Template(#[from] crate::templ::TemplateError),

    #[error("io error: {0}")]
    Fs(#[from] fsutil::FsError),

    #[error("walkdir error: {0}")]
    Walkdir(String),

    #[error("stamp error: {0}")]
    Stamp(#[from] crate::stamps::StampError),

    #[error("missing template_dir for template renderer")]
    MissingTemplateDir,

    #[error("renderer type not implemented in driftx: {0:?}")]
    UnsupportedRenderer(crate::model::RendererType),
}

pub fn diff_plan(repo_root: &Path, plan: &OutputPlan) -> Result<DiffReport, DriftxError> {
    let mut entries = vec![];

    for out in &plan.outputs {
        entries.push(diff_one(repo_root, out)?);
    }

    // Optional: detect generated files that are no longer planned.
    //
    // v1 scope: only files stamped by agents AND matching this plan's adapter id.
    let planned_paths: BTreeSet<String> = plan
        .outputs
        .iter()
        .map(|o| o.path.as_str().to_string())
        .collect();
    let stale = detect_stale_generated(repo_root, &plan.agent_id, &planned_paths)?;
    entries.extend(stale);

    Ok(DiffReport { entries })
}

fn diff_one(repo_root: &Path, out: &PlannedOutput) -> Result<DiffEntry, DriftxError> {
    let target_path = repo_root.join(out.path.as_str());

    // Render planned bytes without stamp.
    let planned_without_stamp = render_planned(out)?;

    // Use drift classification.
    let drift = classify(&target_path, &planned_without_stamp, &out.drift_detection)?;

    // Read existing if present.
    let existing = if target_path.exists() {
        Some(fsutil::read_to_string(&target_path)?)
    } else {
        None
    };

    let existing_without_stamp = existing
        .as_deref()
        .map(|s| strip_existing_stamp(s).0)
        .unwrap_or_default();

    // Decide kind.
    let (kind, details) = match drift {
        DriftStatus::Missing => (DiffKind::Create, None),
        DriftStatus::Unmanaged => (DiffKind::UnmanagedExists, None),
        DriftStatus::Clean => {
            if existing_without_stamp == planned_without_stamp {
                (DiffKind::Noop, None)
            } else {
                // Should be rare: either a stamp mismatch or newline normalization difference.
                (
                    DiffKind::Update,
                    Some("content differs but drift classified clean".to_string()),
                )
            }
        }
        DriftStatus::Drifted => (DiffKind::Drifted, None),
    };

    let unified_diff = match kind {
        DiffKind::Noop => None,
        DiffKind::Create => {
            // Diff against empty.
            Some(unified_diff_for(
                "",
                &planned_without_stamp,
                "(missing)",
                out.path.as_str(),
            ))
        }
        DiffKind::Update | DiffKind::Drifted => Some(unified_diff_for(
            &existing_without_stamp,
            &planned_without_stamp,
            "(existing)",
            out.path.as_str(),
        )),
        DiffKind::UnmanagedExists => Some(unified_diff_for(
            &existing_without_stamp,
            &planned_without_stamp,
            "(unmanaged)",
            out.path.as_str(),
        )),
        DiffKind::Delete => None,
    };

    Ok(DiffEntry {
        path: out.path.as_str().to_string(),
        kind,
        drift: Some(drift),
        details,
        unified_diff,
    })
}

fn render_planned(out: &PlannedOutput) -> Result<String, DriftxError> {
    match out.renderer.type_ {
        crate::model::RendererType::Template => {
            let dir = out
                .template_dir
                .as_ref()
                .ok_or(DriftxError::MissingTemplateDir)?;
            let mut engine = TemplateEngine::new();
            engine.register_partials_from_dir(dir)?;

            let template_name = out.renderer.template.as_deref().unwrap_or("<missing>");

            Ok(engine.render(template_name, &out.render_context)?)
        }
        other => Err(DriftxError::UnsupportedRenderer(other)),
    }
}
