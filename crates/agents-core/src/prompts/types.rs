use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PromptId(pub String);

#[derive(Debug, Clone)]
pub struct Snippet {
    pub id: String,
    pub path: PathBuf,
    pub md: String,
}

#[derive(Debug, Clone)]
pub struct EffectivePrompts {
    pub base_md: String,
    pub project_md: String,
    pub snippets: Vec<Snippet>,
    pub composed_md: String,
}

#[derive(Debug, Clone)]
pub struct PromptSource {
    pub path: PathBuf,
    pub kind: &'static str,
}
