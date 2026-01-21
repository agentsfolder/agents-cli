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
    pub dry_run: bool,
}

pub fn cmd_import(repo_root: &Path, opts: ImportOptions) -> Result<(), AppError> {
    let from = opts.from_agent.trim();
    let inputs = resolve_import_inputs(repo_root, from, opts.path.as_deref())?;

    ensure_agents_not_initialized(repo_root)?;

    let importer: Box<dyn Importer> = match from {
        "copilot" => Box::new(CopilotImporter),
        _ => {
            return Err(AppError {
                category: ErrorCategory::InvalidArgs,
                message: "unsupported import source".to_string(),
                context: vec![
                    format!("from: {from}"),
                    "hint: supported: copilot".to_string(),
                ],
            })
        }
    };

    let artifacts = importer.convert(inputs).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e,
        context: vec![],
    })?;

    if opts.dry_run {
        let mut paths: Vec<&str> = artifacts
            .files
            .iter()
            .map(|f| f.rel_path.as_str())
            .collect();
        paths.sort();
        println!("dry-run: would write {} files", paths.len());
        for p in paths {
            println!("write: {p}");
        }
        return Ok(());
    }

    for f in &artifacts.files {
        write_file(repo_root, &f.rel_path, &f.contents)?;
    }

    // Validate produced artifacts.
    let (cfg, _report) = agents_core::loadag::load_repo_config(
        repo_root,
        &agents_core::loadag::LoaderOptions {
            require_schemas_dir: true,
        },
    )
    .map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    agents_core::schemas::validate_repo_config(repo_root, &cfg).map_err(|err| AppError {
        category: ErrorCategory::SchemaInvalid,
        message: format!("schema invalid: {} ({})", err.path.display(), err.schema),
        context: {
            let mut c = vec![format!("pointer: {}", err.pointer), err.message];
            if let Some(h) = err.hint {
                c.push(h);
            }
            c
        },
    })?;

    println!("ok: imported into .agents/ (from: {})", importer.agent_id());
    println!("ok: schemas valid");
    println!("next: run `agents status` and `agents preview --agent copilot`");

    Ok(())
}

struct CopilotImporter;

impl Importer for CopilotImporter {
    fn agent_id(&self) -> &'static str {
        "copilot"
    }

    fn discover(&self, _repo_root: &Path) -> Option<ImportInputs> {
        None
    }

    fn convert(&self, inputs: ImportInputs) -> Result<CanonicalArtifacts, String> {
        let mut files: Vec<CanonicalFile> = vec![];

        // Start from the standard preset.
        let base =
            crate::initpr::assets::files_for_preset(crate::initpr::assets::InitPreset::Standard);
        for f in base {
            files.push(CanonicalFile {
                rel_path: f.rel_path.to_string(),
                contents: normalize_lf(f.contents).into_bytes(),
            });
        }

        // Ensure Copilot adapter is present.
        files.push(CanonicalFile {
            rel_path: ".agents/adapters/copilot/adapter.yaml".to_string(),
            contents: normalize_lf(include_str!(
                "../initpr/assets/agent-pack/adapters/copilot/adapter.yaml"
            ))
            .into_bytes(),
        });
        files.push(CanonicalFile {
            rel_path: ".agents/adapters/copilot/templates/copilot-instructions.md.hbs".to_string(),
            contents: normalize_lf(include_str!(
                "../initpr/assets/agent-pack/adapters/copilot/templates/copilot-instructions.md.hbs"
            ))
            .into_bytes(),
        });
        files.push(CanonicalFile {
            rel_path: ".agents/adapters/copilot/templates/scope.instructions.md.hbs".to_string(),
            contents: normalize_lf(include_str!(
                "../initpr/assets/agent-pack/adapters/copilot/templates/scope.instructions.md.hbs"
            ))
            .into_bytes(),
        });

        // Replace manifest with one that enables the copilot adapter.
        files.push(CanonicalFile {
            rel_path: ".agents/manifest.yaml".to_string(),
            contents: normalize_lf(COPILOT_IMPORT_MANIFEST).into_bytes(),
        });

        // Import content as a snippet and provide an opt-in mode.
        let mut snippet_md = inputs.content;
        if !snippet_md.ends_with('\n') {
            snippet_md.push('\n');
        }
        files.push(CanonicalFile {
            rel_path: ".agents/prompts/snippets/copilot.md".to_string(),
            contents: normalize_lf(&snippet_md).into_bytes(),
        });

        files.push(CanonicalFile {
            rel_path: ".agents/modes/copilot-import.md".to_string(),
            contents: normalize_lf(COPILOT_IMPORT_MODE).into_bytes(),
        });

        Ok(CanonicalArtifacts { files })
    }
}

const COPILOT_IMPORT_MANIFEST: &str = "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, backend: materialize, sharedSurfacesOwner: core }\n\
enabled: { modes: [default, readonly-audit, copilot-import], policies: [safe, conservative, ci-safe], skills: [], adapters: [core, copilot] }\n";

const COPILOT_IMPORT_MODE: &str = "---\n\
id: copilot-import\n\
title: Copilot Import\n\
includeSnippets: [copilot]\n\
---\n\
\n\
This mode includes imported Copilot instructions as a snippet (copilot).\n\
";

fn ensure_agents_not_initialized(repo_root: &Path) -> Result<(), AppError> {
    let manifest = repo_root.join(".agents/manifest.yaml");
    if manifest.is_file() {
        return Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: "refusing to import: .agents already exists".to_string(),
            context: vec![
                format!("path: {}", manifest.display()),
                "hint: edit .agents manually or remove it before importing".to_string(),
            ],
        });
    }

    let agents_dir = repo_root.join(".agents");
    if !agents_dir.exists() {
        return Ok(());
    }
    if !agents_dir.is_dir() {
        return Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: ".agents exists but is not a directory".to_string(),
            context: vec![format!("path: {}", agents_dir.display())],
        });
    }

    let mut entries: Vec<String> = vec![];
    for entry in std::fs::read_dir(&agents_dir).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", agents_dir.display())],
    })? {
        let entry = entry.map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", agents_dir.display())],
        })?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".DS_Store" {
            continue;
        }
        entries.push(name);
    }

    if !entries.is_empty() {
        entries.sort();
        return Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: "refusing to import: .agents is not empty".to_string(),
            context: vec![
                format!("path: {}", agents_dir.display()),
                format!("entries: {}", entries.join(", ")),
            ],
        });
    }

    Ok(())
}

fn normalize_lf(s: &str) -> String {
    // Avoid CRLF differences in imported content.
    s.replace("\r\n", "\n")
}

fn write_file(repo_root: &Path, rel_path: &str, bytes: &[u8]) -> Result<(), AppError> {
    let dest = repo_root.join(rel_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", parent.display())],
        })?;
    }

    agents_core::fsutil::atomic_write(&dest, bytes).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", dest.display())],
    })?;

    Ok(())
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
