use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::model::{Adapter, Manifest, ModeFile, Policy, Scope, Skill, State};

#[derive(Debug, Clone)]
pub struct PromptLibrary {
    pub base_md: String,
    pub project_md: String,
    pub snippets: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RepoConfig {
    pub repo_root: PathBuf,

    pub manifest: Manifest,

    pub policies: BTreeMap<String, Policy>,
    pub skills: BTreeMap<String, Skill>,
    pub skill_dirs: BTreeMap<String, PathBuf>,

    pub scopes: BTreeMap<String, Scope>,

    pub modes: BTreeMap<String, ModeFile>,

    pub adapters: BTreeMap<String, Adapter>,
    pub adapter_template_dirs: BTreeMap<String, PathBuf>,

    pub profiles: BTreeMap<String, serde_yaml::Value>,

    pub prompts: PromptLibrary,

    pub state: Option<State>,
}
