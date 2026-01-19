use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,

    #[serde(default)]
    pub tags: Vec<String>,

    pub activation: SkillActivation,

    pub interface: SkillInterface,

    pub contract: SkillContract,

    pub requirements: SkillRequirements,

    #[serde(default)]
    pub assets: Option<SkillAssets>,

    #[serde(default)]
    pub compatibility: Option<SkillCompatibility>,

    #[serde(default)]
    pub x: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillActivation {
    InstructionOnly,
    McpTool,
    CliShim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillInterface {
    #[serde(rename = "type")]
    pub type_: SkillInterfaceType,

    #[serde(default)]
    pub entrypoint: Option<String>,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillInterfaceType {
    Mcp,
    Cli,
    Script,
    Library,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillContract {
    pub inputs: serde_json::Value,
    pub outputs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillRequirements {
    pub capabilities: SkillRequiredCapabilities,

    #[serde(default)]
    pub paths: Option<SkillRequiredPaths>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillRequiredCapabilities {
    pub filesystem: RequiredLevel,
    pub exec: RequiredLevel,
    pub network: RequiredLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequiredLevel {
    None,
    Read,
    Write,
    Restricted,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillRequiredPaths {
    #[serde(default)]
    pub needs: Vec<String>,

    #[serde(default)]
    pub writes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillAssets {
    #[serde(default)]
    pub mount: Vec<String>,

    #[serde(default)]
    pub materialize: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCompatibility {
    #[serde(default)]
    pub agents: Vec<String>,

    #[serde(default)]
    pub backends: Vec<crate::model::manifest::BackendKind>,
}
