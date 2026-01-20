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
    ensure_state_gitignore(repo_root)?;

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

fn ensure_state_gitignore(repo_root: &Path) -> Result<(), AppError> {
    let state_dir = agents_core::fsutil::agents_state_dir(repo_root);
    std::fs::create_dir_all(&state_dir).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![format!("path: {}", state_dir.display())],
    })?;

    let p = state_dir.join(".gitignore");
    let mut content = if p.is_file() {
        agents_core::fsutil::read_to_string(&p).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", p.display())],
        })?
    } else {
        String::new()
    };

    let mut changed = false;
    for rule in ["state.yaml", "explain/"] {
        let rooted = format!("/{rule}");
        let has = content
            .lines()
            .any(|l| l.trim() == rule || l.trim() == rooted);
        if !has {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(rule);
            content.push('\n');
            changed = true;
        }
    }

    if changed {
        agents_core::fsutil::atomic_write(&p, content.as_bytes()).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", p.display())],
        })?;
    }

    Ok(())
}

pub fn cmd_explain(repo_root: &Path, input_path: &Path, output: OutputMode) -> Result<(), AppError> {
    let repo_rel = normalize_repo_rel_path(repo_root, input_path)?;
    let p = explain_record_path(repo_root, &repo_rel);

    if !p.is_file() {
        // Fall back to stamp parsing for minimal explanation.
        let abs = repo_root.join(&repo_rel);
        if abs.is_file() {
            let content = std::fs::read_to_string(&abs).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![format!("path: {}", abs.display())],
            })?;

            if let Some(stamp) = agents_core::stamps::parse_stamp(&content) {
                return Ok(print_stamp_explain(&repo_rel, &stamp, output));
            }

            return Err(AppError {
                category: ErrorCategory::Io,
                message: "unmanaged file (no agents stamp and no source map)".to_string(),
                context: vec![
                    format!("path: {repo_rel}"),
                    "hint: only stamped outputs written by agents (or outputs planned during preview/sync) can be explained"
                        .to_string(),
                ],
            });
        }

        return Err(AppError {
            category: ErrorCategory::Io,
            message: "file not found".to_string(),
            context: vec![format!("path: {repo_rel}")],
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

fn print_stamp_explain(repo_rel: &str, stamp: &agents_core::stamps::Stamp, output: OutputMode) {
    match output {
        OutputMode::Json => {
            #[derive(serde::Serialize)]
            struct StampExplain<'a> {
                path: &'a str,
                stamp: &'a agents_core::stamps::Stamp,
            }

            let s = serde_json::to_string_pretty(&StampExplain { path: repo_rel, stamp })
                .unwrap_or_else(|_| "{}".to_string());
            println!("{s}");
        }
        OutputMode::Human => {
            println!("path: {repo_rel}");
            println!("generator: {}", stamp.meta.generator);
            println!("adapter: {}", stamp.meta.adapter_agent_id);
            println!("mode: {}", stamp.meta.mode);
            println!("policy: {}", stamp.meta.policy);
            println!("backend: {:?}", stamp.meta.backend);
            println!(
                "profile: {}",
                stamp.meta.profile.as_deref().unwrap_or("<none>")
            );
        }
    }
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
    if !m.renderer.sources.is_empty() {
        println!("sources:");
        for s in &m.renderer.sources {
            println!("- {s}");
        }
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
