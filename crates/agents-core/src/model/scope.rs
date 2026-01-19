use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub id: String,

    #[serde(rename = "applyTo")]
    pub apply_to: Vec<String>,

    #[serde(default)]
    pub priority: i64,

    pub overrides: ScopeOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeOverrides {
    #[serde(default)]
    pub mode: Option<String>,

    #[serde(default)]
    pub policy: Option<String>,

    #[serde(default, rename = "enableSkills")]
    pub enable_skills: Vec<String>,

    #[serde(default, rename = "disableSkills")]
    pub disable_skills: Vec<String>,

    #[serde(default, rename = "includeSnippets")]
    pub include_snippets: Vec<String>,
}
