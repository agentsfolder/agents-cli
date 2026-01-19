use serde::{Deserialize, Serialize};

use crate::model::manifest::BackendKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    #[serde(rename = "agentId")]
    pub agent_id: String,

    pub version: String,

    #[serde(rename = "backendDefaults")]
    pub backend_defaults: BackendDefaults,

    #[serde(default, rename = "capabilityMapping")]
    pub capability_mapping: Option<serde_json::Value>,

    pub outputs: Vec<AdapterOutput>,

    #[serde(default)]
    pub tests: Option<AdapterTests>,

    #[serde(default)]
    pub x: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendDefaults {
    pub preferred: BackendKind,
    pub fallback: BackendKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterOutput {
    pub path: String,

    #[serde(default)]
    pub format: Option<OutputFormat>,

    #[serde(default)]
    pub surface: Option<String>,

    #[serde(default)]
    pub collision: Option<CollisionPolicy>,

    #[serde(default)]
    pub condition: Option<OutputCondition>,

    pub renderer: OutputRenderer,

    #[serde(default, rename = "writePolicy")]
    pub write_policy: Option<WritePolicy>,

    #[serde(default, rename = "driftDetection")]
    pub drift_detection: Option<DriftDetection>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Md,
    Yaml,
    Json,
    Jsonc,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollisionPolicy {
    Error,
    Overwrite,
    Merge,
    SharedOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputCondition {
    #[serde(default, rename = "backendIn")]
    pub backend_in: Vec<BackendKind>,

    #[serde(default, rename = "profileIn")]
    pub profile_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputRenderer {
    #[serde(rename = "type")]
    pub type_: RendererType,

    #[serde(default)]
    pub template: Option<String>,

    #[serde(default)]
    pub sources: Vec<String>,

    #[serde(default, rename = "jsonMergeStrategy")]
    pub json_merge_strategy: Option<JsonMergeStrategy>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RendererType {
    Template,
    Concat,
    Copy,
    JsonMerge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JsonMergeStrategy {
    Deep,
    Shallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WritePolicy {
    #[serde(default)]
    pub mode: Option<WriteMode>,

    #[serde(default)]
    pub gitignore: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    Always,
    IfGenerated,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftDetection {
    #[serde(default)]
    pub method: Option<DriftMethod>,

    #[serde(default)]
    pub stamp: Option<StampMethod>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftMethod {
    Sha256,
    MtimeOnly,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StampMethod {
    Comment,
    Frontmatter,
    JsonField,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterTests {
    #[serde(default, rename = "goldenFixturesDir")]
    pub golden_fixtures_dir: Option<String>,

    #[serde(default, rename = "goldenCommand")]
    pub golden_command: Option<String>,
}
