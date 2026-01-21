use std::path::Path;

pub mod assets;

use crate::{AppError, ErrorCategory};

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub preset: Option<String>,
}

pub fn cmd_init(repo_root: &Path, opts: InitOptions) -> Result<(), AppError> {
    let preset = opts
        .preset
        .as_deref()
        .unwrap_or("standard")
        .trim()
        .to_string();

    let preset = assets::InitPreset::parse(&preset).ok_or_else(|| AppError {
        category: ErrorCategory::InvalidArgs,
        message: "unknown init preset".to_string(),
        context: vec![
            format!("preset: {preset}"),
            "allowed: conservative, standard, ci-safe, monorepo, agent-pack".to_string(),
        ],
    })?;

    ensure_agents_dir_empty(repo_root)?;

    let files = assets::files_for_preset(preset);
    for f in &files {
        write_embedded_file(repo_root, f)?;
    }

    // Post-init validation: mirror `agents validate` behavior (fail if invalid).
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

    println!("ok: initialized .agents/ (preset: {})", preset.as_str());
    println!("ok: schemas valid");
    println!("next: run `agents status` and `agents preview`");

    Ok(())
}

fn ensure_agents_dir_empty(repo_root: &Path) -> Result<(), AppError> {
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
        let name = entry
            .file_name()
            .to_string_lossy()
            .to_string();
        if name == ".DS_Store" {
            continue;
        }
        entries.push(name);
    }

    if !entries.is_empty() {
        entries.sort();
        return Err(AppError {
            category: ErrorCategory::InvalidArgs,
            message: "refusing to initialize: .agents is not empty".to_string(),
            context: vec![
                format!("path: {}", agents_dir.display()),
                format!("entries: {}", entries.join(", ")),
            ],
        });
    }

    Ok(())
}

fn write_embedded_file(repo_root: &Path, f: &assets::EmbeddedFile) -> Result<(), AppError> {
    let dest = repo_root.join(f.rel_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", parent.display())],
        })?;
    }

    agents_core::fsutil::atomic_write(&dest, f.contents.as_bytes()).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", dest.display())],
    })?;

    Ok(())
}
