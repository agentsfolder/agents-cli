use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{BackendKind, CollisionPolicy, OutputFormat, RendererType};
use crate::outputs::{OutputPlan, SourceMapSkeleton};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainSourceMap {
    pub output_path: String,
    pub surface: Option<String>,

    pub adapter_id: String,
    pub output_format: OutputFormat,
    pub collision: CollisionPolicy,
    pub renderer: ExplainRenderer,

    pub effective: ExplainEffectiveConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainRenderer {
    #[serde(rename = "type")]
    pub type_: RendererType,
    pub template: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainEffectiveConfig {
    pub mode_id: String,
    pub policy_id: String,
    pub profile: Option<String>,
    pub backend: BackendKind,

    #[serde(default)]
    pub scopes_matched: Vec<String>,

    #[serde(default)]
    pub prompt_source_paths: Vec<String>,

    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub snippet_ids: Vec<String>,
}

pub fn build_explain_source_maps(
    plan: &OutputPlan,
    sources: &[SourceMapSkeleton],
) -> Vec<ExplainSourceMap> {
    let mut by_output: BTreeMap<&str, &SourceMapSkeleton> = BTreeMap::new();
    for s in sources {
        by_output.insert(&s.output_path, s);
    }

    let mut out: Vec<ExplainSourceMap> = vec![];
    for p in &plan.outputs {
        let skel = by_output.get(p.path.as_str()).copied();

        out.push(ExplainSourceMap {
            output_path: p.path.as_str().to_string(),
            surface: p.surface.clone(),
            adapter_id: plan.agent_id.clone(),
            output_format: p.format,
            collision: p.collision,
            renderer: ExplainRenderer {
                type_: p.renderer.type_,
                template: p.renderer.template.clone(),
                sources: p.renderer.sources.clone(),
            },
            effective: ExplainEffectiveConfig {
                mode_id: skel.map(|s| s.mode_id.clone()).unwrap_or_default(),
                policy_id: skel.map(|s| s.policy_id.clone()).unwrap_or_default(),
                profile: p.render_context.profile.clone(),
                backend: plan.backend,
                scopes_matched: p.render_context.scopes_matched.clone(),
                prompt_source_paths: skel
                    .map(|s| s.prompt_source_paths.clone())
                    .unwrap_or_default(),
                skill_ids: skel.map(|s| s.skill_ids.clone()).unwrap_or_default(),
                snippet_ids: skel.map(|s| s.snippet_ids.clone()).unwrap_or_default(),
            },
        });
    }

    out
}
