use std::path::{Path, PathBuf};

use crate::fsutil;
use crate::model::OutputFormat;
use crate::outputs::OutputPlan;

use super::{ApplyReport, Backend, BackendError, BackendSession, RenderedOutput};

#[derive(Debug, Default, Clone)]
pub struct MaterializeBackend;

impl Backend for MaterializeBackend {
    fn prepare(&self, repo_root: &Path, plan: &OutputPlan) -> Result<BackendSession, BackendError> {
        Ok(BackendSession {
            repo_root: repo_root.to_path_buf(),
            plan: plan.clone(),
        })
    }

    fn apply(
        &self,
        session: &mut BackendSession,
        outputs: &[RenderedOutput],
    ) -> Result<ApplyReport, BackendError> {
        let mut report = ApplyReport::default();

        for out in outputs {
            let dest = session.repo_root.join(out.path.as_str());
            let bytes = normalize_bytes_for_write(&dest, &out.bytes, None);

            fsutil::atomic_write(&dest, &bytes)?;
            report.written.push(out.path.clone());
        }

        Ok(report)
    }
}

fn normalize_bytes_for_write(_path: &PathBuf, bytes: &[u8], format: Option<OutputFormat>) -> Vec<u8> {
    let format = format.unwrap_or(OutputFormat::Text);
    if !is_text_format(format) {
        return bytes.to_vec();
    }

    // Best-effort: if content is UTF-8, normalize newlines + trailing newline.
    let Ok(s) = std::str::from_utf8(bytes) else {
        return bytes.to_vec();
    };

    let normalized = s.replace("\r\n", "\n");
    fsutil::ensure_trailing_newline(&normalized).into_bytes()
}

fn is_text_format(format: OutputFormat) -> bool {
    match format {
        OutputFormat::Text | OutputFormat::Md | OutputFormat::Yaml | OutputFormat::Json | OutputFormat::Jsonc => true,
    }
}
