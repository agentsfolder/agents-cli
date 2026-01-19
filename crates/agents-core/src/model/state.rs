use serde::{Deserialize, Serialize};

use crate::model::manifest::BackendKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub mode: String,

    #[serde(default)]
    pub profile: Option<String>,

    #[serde(default)]
    pub backend: Option<BackendKind>,

    #[serde(default)]
    pub scopes: Vec<String>,
}
