use serde::{Deserialize, Serialize};

use crate::model::manifest::BackendKind;
use crate::model::StampMethod;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StampMeta {
    pub generator: String,

    #[serde(rename = "adapterAgentId")]
    pub adapter_agent_id: String,

    #[serde(rename = "manifestSpecVersion")]
    pub manifest_spec_version: String,

    pub mode: String,
    pub policy: String,
    pub backend: BackendKind,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,

    #[serde(rename = "contentSha256")]
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub method: StampMethod,
    pub meta: StampMeta,
}
