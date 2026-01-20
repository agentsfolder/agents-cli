use std::path::{Path, PathBuf};

use agents_core::explain::build_explain_source_maps;
use agents_core::outputs::PlanResult;
use agents_core::stamps::compute_sha256_hex;

use crate::{AppError, ErrorCategory, OutputMode};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ExplainRecord {
    version: u32,
    map: agents_core::explain::ExplainSourceMap,
}

pub fn persist_source_maps(repo_root: &Path, plan_res: &PlanResult) -> Result<(), AppError> {
    let dir = agents_core::fsutil::agents_explain_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", dir.display())],
    })?;

    let maps = build_explain_source_maps(&plan_res.plan, &plan_res.sources);
    for m in maps {
        let hash = compute_sha256_hex(m.output_path.as_str());
        let dest: PathBuf = dir.join(format!("{hash}.json"));
        let rec = ExplainRecord { version: 1, map: m };

        let bytes = serde_json::to_vec_pretty(&rec).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec!["while serializing explain source map".to_string()],
        })?;

        agents_core::fsutil::atomic_write(&dest, &bytes).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", dest.display())],
        })?;
    }

    Ok(())
}

pub fn cmd_explain(repo_root: &Path, input_path: &Path, output: OutputMode) -> Result<(), AppError> {
    let repo_rel = normalize_repo_rel_path(repo_root, input_path)?;
    let p = explain_record_path(repo_root, &repo_rel);

    if !p.is_file() {
        return Err(AppError {
            category: ErrorCategory::Io,
            message: "no explain source map found".to_string(),
            context: vec![
                format!("path: {repo_rel}"),
                "hint: run `agents preview` or `agents sync` first".to_string(),
            ],
        });
    }

    let bytes = std::fs::read(&p).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", p.display())],
    })?;

    let rec: ExplainRecord = serde_json::from_slice(&bytes).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", p.display())],
    })?;

    match output {
        OutputMode::Json => {
            let s = serde_json::to_string_pretty(&rec).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;
            println!("{s}");
        }
        OutputMode::Human => {
            print_explain_human(&rec.map);
        }
    }

    Ok(())
}

fn normalize_repo_rel_path(repo_root: &Path, input: &Path) -> Result<String, AppError> {
    let rel = if input.is_absolute() {
        input.strip_prefix(repo_root).map_err(|_| AppError {
            category: ErrorCategory::InvalidArgs,
            message: "path must be inside repo".to_string(),
            context: vec![format!("path: {}", input.display())],
        })?
    } else {
        input
    };

    let rp = agents_core::fsutil::repo_relpath_noexist(repo_root, rel).map_err(|e| AppError {
        category: ErrorCategory::InvalidArgs,
        message: e.to_string(),
        context: vec![format!("path: {}", input.display())],
    })?;

    Ok(rp.as_str().to_string())
}

fn explain_record_path(repo_root: &Path, repo_rel: &str) -> PathBuf {
    let hash = compute_sha256_hex(repo_rel);
    agents_core::fsutil::agents_explain_dir(repo_root).join(format!("{hash}.json"))
}

fn print_explain_human(m: &agents_core::explain::ExplainSourceMap) {
    println!("path: {}", m.output_path);
    println!("adapter: {}", m.adapter_id);
    if let Some(surface) = &m.surface {
        println!("surface: {surface}");
    }

    println!("format: {:?}", m.output_format);
    println!("collision: {:?}", m.collision);
    println!("renderer: {:?}", m.renderer.type_);
    if let Some(t) = &m.renderer.template {
        println!("template: {t}");
    }

    println!("mode: {}", m.effective.mode_id);
    println!("policy: {}", m.effective.policy_id);
    println!("backend: {:?}", m.effective.backend);
    println!(
        "profile: {}",
        m.effective.profile.as_deref().unwrap_or("<none>")
    );

    if !m.effective.scopes_matched.is_empty() {
        println!("scopes: {}", m.effective.scopes_matched.join(", "));
    }
    if !m.effective.skill_ids.is_empty() {
        println!("skills: {}", m.effective.skill_ids.join(", "));
    }
    if !m.effective.snippet_ids.is_empty() {
        println!("snippets: {}", m.effective.snippet_ids.join(", "));
    }

    if !m.effective.prompt_source_paths.is_empty() {
        println!("prompt_sources:");
        for p in &m.effective.prompt_source_paths {
            println!("- {p}");
        }
    }
}
