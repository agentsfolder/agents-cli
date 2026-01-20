use std::path::PathBuf;

use crate::fsutil::RepoPath;
use crate::model::{CollisionPolicy, DriftDetection, OutputFormat, OutputRenderer, WritePolicy};
use crate::templ::RenderContext;

#[derive(Debug, Clone)]
pub struct PlannedOutput {
    pub path: RepoPath,
    pub format: OutputFormat,
    pub surface: Option<String>,
    pub collision: CollisionPolicy,
    pub renderer: OutputRenderer,
    pub write_policy: WritePolicy,
    pub drift_detection: DriftDetection,

    /// Optional resolved template dir for convenience.
    pub template_dir: Option<PathBuf>,

    pub render_context: RenderContext,
}

#[derive(Debug, Clone)]
pub struct OutputPlan {
    pub agent_id: String,
    pub backend: crate::model::BackendKind,
    pub outputs: Vec<PlannedOutput>,
}

#[derive(Debug, Clone)]
pub struct SourceMapSkeleton {
    pub adapter_id: String,
    pub output_path: String,
    pub template: Option<String>,

    /// Repo-relative prompt/snippet file paths that contributed to the effective prompt.
    pub prompt_source_paths: Vec<String>,

    pub mode_id: String,
    pub policy_id: String,
    pub skill_ids: Vec<String>,
    pub snippet_ids: Vec<String>,
}
