use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(".agents is not initialized: missing {path}")]
    NotInitialized { path: PathBuf },

    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("parse error at {path}: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("duplicate id {id} in {kind}")]
    DuplicateId { kind: &'static str, id: String },

    #[error("missing required id {id} in {kind}")]
    MissingId { kind: &'static str, id: String },
}

#[derive(Debug, Clone)]
pub struct LoadWarning {
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LoadReport {
    pub warnings: Vec<LoadWarning>,
}

impl LoadReport {
    pub fn new() -> Self {
        Self { warnings: vec![] }
    }

    pub fn warn(&mut self, path: Option<PathBuf>, message: impl Into<String>) {
        self.warnings.push(LoadWarning {
            path,
            message: message.into(),
        });
    }
}
