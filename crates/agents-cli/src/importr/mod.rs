use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImportInputs {
    pub source_path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CanonicalFile {
    pub rel_path: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CanonicalArtifacts {
    pub files: Vec<CanonicalFile>,
}

pub trait Importer {
    fn agent_id(&self) -> &'static str;

    fn discover(&self, repo_root: &Path) -> Option<ImportInputs>;

    fn convert(&self, inputs: ImportInputs) -> Result<CanonicalArtifacts, String>;
}
