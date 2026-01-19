use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::loadag::RepoConfig;
use crate::model::Policy;
use crate::prompts::{EffectivePrompts, PromptSource, Snippet};
use crate::resolv::EffectiveConfig;

#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("unknown snippet id: {id}")]
    UnknownSnippet { id: String },

    #[error("invalid redact glob: {glob}: {message}")]
    InvalidGlob { glob: String, message: String },
}

#[derive(Debug, Clone)]
pub struct PromptComposer {
    repo_root: PathBuf,
    repo: RepoConfig,
}

impl PromptComposer {
    pub fn new(repo_root: &Path, repo: RepoConfig) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            repo,
        }
    }

    pub fn compose(
        &self,
        effective: &EffectiveConfig,
        policy: &Policy,
    ) -> Result<(EffectivePrompts, Vec<PromptSource>), PromptError> {
        let snippet_ids = self.collect_snippet_ids(effective);

        let mut snippets: Vec<Snippet> = Vec::with_capacity(snippet_ids.len());
        for id in snippet_ids {
            let md = self
                .repo
                .prompts
                .snippets
                .get(&id)
                .cloned()
                .ok_or(PromptError::UnknownSnippet { id: id.clone() })?;

            snippets.push(Snippet {
                id: id.clone(),
                path: self
                    .repo_root
                    .join(".agents/prompts/snippets")
                    .join(format!("{id}.md")),
                md,
            });
        }

        let base_md = normalize_newlines(&self.repo.prompts.base_md);
        let project_md = normalize_newlines(&self.repo.prompts.project_md);

        let composed_md = compose_markdown(&base_md, &project_md, &snippets);

        // Build stable sources list.
        let mut sources: Vec<PromptSource> = vec![
            PromptSource {
                path: self.repo_root.join(".agents/prompts/base.md"),
                kind: "base",
            },
            PromptSource {
                path: self.repo_root.join(".agents/prompts/project.md"),
                kind: "project",
            },
        ];

        for snip in &snippets {
            sources.push(PromptSource {
                path: snip.path.clone(),
                kind: "snippet",
            });
        }

        // Redaction helper is exposed for later usage; prompt composition itself does not embed file contents.
        let _redactor = Redactor::from_policy(policy)?;

        Ok((
            EffectivePrompts {
                base_md,
                project_md,
                snippets,
                composed_md,
            },
            sources,
        ))
    }

    fn collect_snippet_ids(&self, effective: &EffectiveConfig) -> Vec<String> {
        let mut set: BTreeSet<String> = BTreeSet::new();

        // From resolver (mode + scopes).
        for id in &effective.snippet_ids_included {
            set.insert(id.clone());
        }

        set.into_iter().collect()
    }
}

fn compose_markdown(base: &str, project: &str, snippets: &[Snippet]) -> String {
    let mut out = String::new();

    push_section(&mut out, base);
    push_section(&mut out, project);

    for snip in snippets {
        push_section(&mut out, &snip.md);
    }

    ensure_trailing_newline(&out)
}

fn push_section(out: &mut String, section: &str) {
    if section.trim().is_empty() {
        return;
    }

    if !out.is_empty() {
        // Exactly one blank line between sections.
        if !out.ends_with("\n\n") {
            if out.ends_with('\n') {
                out.push('\n');
            } else {
                out.push_str("\n\n");
            }
        }
    }

    out.push_str(section.trim_end_matches('\n'));
    out.push('\n');
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn ensure_trailing_newline(s: &str) -> String {
    if s.ends_with('\n') {
        s.to_string()
    } else {
        format!("{}\n", s)
    }
}

#[derive(Debug, Clone)]
pub struct Redactor {
    set: GlobSet,
    placeholder: &'static str,
}

impl Redactor {
    pub fn from_policy(policy: &Policy) -> Result<Self, PromptError> {
        let mut builder = GlobSetBuilder::new();
        for pat in &policy.paths.redact {
            let glob = Glob::new(pat).map_err(|e| PromptError::InvalidGlob {
                glob: pat.clone(),
                message: e.to_string(),
            })?;
            builder.add(glob);
        }

        let set = builder.build().map_err(|e| PromptError::InvalidGlob {
            glob: "<set>".to_string(),
            message: e.to_string(),
        })?;

        Ok(Self {
            set,
            placeholder: "[REDACTED]",
        })
    }

    pub fn is_redacted(&self, repo_relative_path: &str) -> bool {
        self.set.is_match(repo_relative_path)
    }

    pub fn placeholder(&self) -> &'static str {
        self.placeholder
    }
}
