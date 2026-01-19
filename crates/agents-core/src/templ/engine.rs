use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;

use crate::fsutil;
use crate::templ::helpers::register_helpers;
use crate::templ::types::RenderContext;

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("io error: {path}: {message}")]
    Io { path: PathBuf, message: String },

    #[error("template render error: {message}")]
    Render { message: String },
}

pub struct TemplateEngine {
    hb: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        register_helpers(&mut hb);

        Self { hb }
    }

    pub fn register_partials_from_dir(
        &mut self,
        templates_dir: &Path,
    ) -> Result<(), TemplateError> {
        if !templates_dir.is_dir() {
            return Ok(());
        }

        let mut files: Vec<PathBuf> = vec![];
        for entry in walk_template_files(templates_dir)? {
            files.push(entry);
        }
        files.sort();

        for path in files {
            let rel = path.strip_prefix(templates_dir).unwrap();
            let name = rel.to_string_lossy().replace('\\', "/");
            let content = fsutil::read_to_string(&path).map_err(|e| TemplateError::Io {
                path: path.clone(),
                message: e.to_string(),
            })?;

            self.hb
                .register_template_string(&name, content)
                .map_err(|e| TemplateError::Render {
                    message: e.to_string(),
                })?;
        }

        Ok(())
    }

    pub fn render(
        &self,
        template_name: &str,
        ctx: &RenderContext,
    ) -> Result<String, TemplateError> {
        let s = self
            .hb
            .render(template_name, ctx)
            .map_err(|e| TemplateError::Render {
                message: e.to_string(),
            })?;

        Ok(normalize_output(&s))
    }

    pub fn render_inline(
        &self,
        template: &str,
        ctx: &RenderContext,
    ) -> Result<String, TemplateError> {
        let s = self
            .hb
            .render_template(template, ctx)
            .map_err(|e| TemplateError::Render {
                message: e.to_string(),
            })?;

        Ok(normalize_output(&s))
    }
}

fn walk_template_files(dir: &Path) -> Result<Vec<PathBuf>, TemplateError> {
    let mut out = vec![];

    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let entries = std::fs::read_dir(&cur).map_err(|e| TemplateError::Io {
            path: cur.clone(),
            message: e.to_string(),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| TemplateError::Io {
                path: cur.clone(),
                message: e.to_string(),
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }

    Ok(out)
}

fn normalize_output(s: &str) -> String {
    // Normalize newlines
    let mut out = s.replace("\r\n", "\n");

    // Ensure trailing newline for text/markdown usage.
    if !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

#[allow(dead_code)]
fn stable_map<K: Ord, V>(pairs: Vec<(K, V)>) -> BTreeMap<K, V> {
    pairs.into_iter().collect()
}
