use std::path::{Path, PathBuf};

use agents_core::explain::build_explain_source_maps;
use agents_core::outputs::PlanResult;
use agents_core::stamps::compute_sha256_hex;

use crate::{AppError, ErrorCategory};

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
