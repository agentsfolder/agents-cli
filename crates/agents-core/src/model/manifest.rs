use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "specVersion")]
    pub spec_version: String,

    #[serde(default)]
    pub project: Option<Project>,

    pub defaults: Defaults,
    pub enabled: Enabled,

    #[serde(default)]
    pub resolution: Option<Resolution>,

    #[serde(default)]
    pub backends: Option<Backends>,

    #[serde(default)]
    pub x: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub mode: String,
    pub policy: String,

    #[serde(default)]
    pub profile: Option<String>,

    #[serde(default)]
    pub backend: Option<BackendKind>,

    #[serde(default, rename = "sharedSurfacesOwner")]
    pub shared_surfaces_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enabled {
    pub modes: Vec<String>,
    pub policies: Vec<String>,
    pub skills: Vec<String>,
    pub adapters: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    VfsContainer,
    Materialize,
    VfsMount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resolution {
    #[serde(default, rename = "enableUserOverlay")]
    pub enable_user_overlay: bool,

    #[serde(default, rename = "denyOverridesAllow")]
    pub deny_overrides_allow: bool,

    #[serde(default, rename = "onConflict")]
    pub on_conflict: Option<OnConflict>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OnConflict {
    Error,
    Warn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Backends {
    #[serde(default)]
    pub default: Option<BackendKind>,

    #[serde(default, rename = "byAgent")]
    pub by_agent: std::collections::BTreeMap<String, BackendKind>,
}
