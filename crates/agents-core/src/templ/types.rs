use std::collections::BTreeMap;

use serde::Serialize;

use crate::model::{ModeFrontmatter, Policy};
use crate::prompts::EffectivePrompts;

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveModeCtx {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<ModeFrontmatter>,

    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveSkillsCtx {
    pub ids: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub summaries: Vec<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationStampCtx {
    pub generator: String,
    pub adapter_agent_id: String,
    pub mode: String,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdapterCtx {
    #[serde(rename = "agentId")]
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveCtx {
    pub mode: EffectiveModeCtx,
    pub policy: Policy,
    pub skills: EffectiveSkillsCtx,
    pub prompts: EffectivePrompts,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderContext {
    pub effective: EffectiveCtx,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    #[serde(rename = "scopesMatched")]
    pub scopes_matched: Vec<String>,

    pub generation: GenerationCtx,

    pub adapter: AdapterCtx,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationCtx {
    pub stamp: GenerationStampCtx,
}
