use std::path::Path;

use crate::fsutil;
use crate::model::{OutputFormat, RendererType, StampMethod};
use crate::outputs::PlannedOutput;
use crate::stamps::{apply_stamp, compute_sha256_hex, StampMeta};
use crate::templ::TemplateEngine;

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("template error: {0}")]
    Template(#[from] crate::templ::TemplateError),

    #[error("fs error: {0}")]
    Fs(#[from] fsutil::FsError),

    #[error("stamp error: {0}")]
    Stamp(#[from] crate::stamps::StampError),

    #[error("missing template_dir for template renderer")]
    MissingTemplateDir,

    #[error("renderer not implemented: {0:?}")]
    UnsupportedRenderer(RendererType),
}

pub struct RenderedOutput {
    pub content_without_stamp: String,
    pub content_with_stamp: String,
    pub output_format: OutputFormat,
}

pub fn render_planned_output(
    _repo_root: &Path,
    out: &PlannedOutput,
) -> Result<RenderedOutput, RenderError> {
    // v1: template only (concat/copy/json_merge added later).
    let content_without_stamp = match out.renderer.type_ {
        RendererType::Template => {
            let mut engine = TemplateEngine::new();

            if let Some(inline) = out.inline_template.as_deref() {
                engine.render_inline(inline, &out.render_context)?
            } else {
                let dir = out
                    .template_dir
                    .as_ref()
                    .ok_or(RenderError::MissingTemplateDir)?;
                engine.register_partials_from_dir(dir)?;

                let template_name = out.renderer.template.as_deref().unwrap_or("");
                engine.render(template_name, &out.render_context)?
            }
        }
        other => return Err(RenderError::UnsupportedRenderer(other)),
    };

    let stamp_method = out.drift_detection.stamp.unwrap_or(StampMethod::Comment);

    let meta = StampMeta {
        generator: "agents".to_string(),
        adapter_agent_id: out.render_context.adapter.agent_id.clone(),
        // v1: use manifest spec version once it is threaded through plan.
        manifest_spec_version: "0.1".to_string(),
        mode: out.render_context.generation.stamp.mode.clone(),
        policy: out.render_context.effective.policy.id.clone(),
        backend: crate::model::manifest::BackendKind::VfsContainer,
        profile: out.render_context.profile.clone(),
        content_sha256: compute_sha256_hex(&content_without_stamp),
    };

    let content_with_stamp = apply_stamp(&content_without_stamp, &meta, stamp_method)?;

    Ok(RenderedOutput {
        content_without_stamp,
        content_with_stamp,
        output_format: out.format,
    })
}
