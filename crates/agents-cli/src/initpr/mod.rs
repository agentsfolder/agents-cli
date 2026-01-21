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

    println!("ok: initialized .agents/ (preset: {})", preset.as_str());
    println!("next: run `agents validate` and `agents status`");

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
