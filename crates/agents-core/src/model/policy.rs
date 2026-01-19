use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub id: String,
    pub description: String,

    pub capabilities: Capabilities,

    pub paths: Paths,

    pub confirmations: Confirmations,

    #[serde(default)]
    pub limits: Option<Limits>,

    #[serde(default)]
    pub x: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default)]
    pub filesystem: Option<FilesystemCaps>,

    #[serde(default)]
    pub exec: Option<ExecCaps>,

    #[serde(default)]
    pub network: Option<NetworkCaps>,

    #[serde(default)]
    pub mcp: Option<McpCaps>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemCaps {
    #[serde(default = "default_true")]
    pub read: bool,

    #[serde(default = "default_true")]
    pub write: bool,

    #[serde(default)]
    pub delete: bool,

    #[serde(default)]
    pub rename: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecCaps {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub allow: Vec<String>,

    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkCaps {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default, rename = "allowHosts")]
    pub allow_hosts: Vec<String>,

    #[serde(default, rename = "denyHosts")]
    pub deny_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpCaps {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default, rename = "allowServers")]
    pub allow_servers: Vec<String>,

    #[serde(default, rename = "denyServers")]
    pub deny_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    #[serde(default)]
    pub allow: Vec<String>,

    #[serde(default)]
    pub deny: Vec<String>,

    #[serde(default)]
    pub redact: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmations {
    #[serde(default, rename = "requiredFor")]
    pub required_for: Vec<ConfirmationType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationType {
    Delete,
    Overwrite,
    Publish,
    Deploy,
    Push,
    Rebase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    #[serde(default, rename = "maxFilesChanged")]
    pub max_files_changed: Option<i64>,

    #[serde(default, rename = "maxPatchLines")]
    pub max_patch_lines: Option<i64>,

    #[serde(default, rename = "maxCommandRuntimeSec")]
    pub max_command_runtime_sec: Option<i64>,
}

fn default_true() -> bool {
    true
}
