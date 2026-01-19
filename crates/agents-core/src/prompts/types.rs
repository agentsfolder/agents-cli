use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct PromptId(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct Snippet {
    pub id: String,

    // Keep paths as display strings in templates (stable across platforms after normalization
    // is handled by upstream layers).
    pub path: String,

    pub md: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectivePrompts {
    pub base_md: String,
    pub project_md: String,
    pub snippets: Vec<Snippet>,
    pub composed_md: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptSource {
    pub path: String,
    pub kind: &'static str,
}
