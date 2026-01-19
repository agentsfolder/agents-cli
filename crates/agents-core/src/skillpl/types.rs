use std::path::PathBuf;

use crate::model::{BackendKind, Skill};

#[derive(Debug, Clone)]
pub struct SkillRef {
    pub id: String,
    pub dir: PathBuf,
    pub skill: Skill,
}

#[derive(Debug, Clone)]
pub struct SkillRequirementsSummary {
    pub filesystem: String,
    pub exec: String,
    pub network: String,

    pub needs_paths: Vec<String>,
    pub writes_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EffectiveSkills {
    pub enabled: Vec<SkillRef>,
    pub disabled: Vec<String>,
    pub warnings: Vec<String>,

    pub backend: BackendKind,
    pub agent_id: Option<String>,
}
