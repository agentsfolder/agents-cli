use std::path::{Path, PathBuf};

use crate::{AppError, ErrorCategory};

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

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub from_agent: String,
    pub path: Option<PathBuf>,
}

pub fn cmd_import(repo_root: &Path, opts: ImportOptions) -> Result<(), AppError> {
    let from = opts.from_agent.trim();
    let inputs = resolve_import_inputs(repo_root, from, opts.path.as_deref())?;

    // Implemented in later steps (see feat-importr): convert + write artifacts.
    Err(AppError {
        category: ErrorCategory::InvalidArgs,
        message: "import not implemented yet".to_string(),
        context: vec![format!("from: {from}"), format!("path: {}", inputs.source_path.display())],
    })
}

fn resolve_import_inputs(
    repo_root: &Path,
    from_agent: &str,
    path_override: Option<&Path>,
) -> Result<ImportInputs, AppError> {
    match from_agent {
        "copilot" => {
            let p = path_override
                .map(|p| {
                    if p.is_absolute() {
                        p.to_path_buf()
                    } else {
                        repo_root.join(p)
                    }
                })
                .unwrap_or_else(|| repo_root.join(".github/copilot-instructions.md"));

            if !p.is_file() {
                return Err(AppError {
                    category: ErrorCategory::InvalidArgs,
                    message: "copilot instructions file not found".to_string(),
                    context: vec![
                        format!("path: {}", p.display()),
                        "hint: pass --path to point at a copilot instructions file".to_string(),
                    ],
                });
            }

            let content = agents_core::fsutil::read_to_string(&p).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![format!("path: {}", p.display())],
            })?;

            Ok(ImportInputs {
                source_path: p,
                content,
            })
        }
        other => Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: "unsupported import source".to_string(),
            context: vec![
                format!("from: {other}"),
                "hint: supported: copilot".to_string(),
            ],
        }),
    }
}
