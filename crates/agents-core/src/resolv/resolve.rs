use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::loadag::RepoConfig;
use crate::model::BackendKind;
use crate::resolv::{match_scopes, EffectiveConfig, ResolutionRequest, ScopeMatch};

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("missing id {id} in {kind}")]
    MissingId { kind: &'static str, id: String },
}

#[derive(Debug)]
pub struct Resolver {
    pub repo: RepoConfig,
}

impl Resolver {
    pub fn new(repo: RepoConfig) -> Self {
        Self { repo }
    }

    pub fn resolve(&self, req: &ResolutionRequest) -> Result<EffectiveConfig, ResolveError> {
        let target_path = req.target_path.clone().unwrap_or_else(|| ".".to_string());

        let scopes_matched = if req.override_scopes.is_empty() {
            match_scopes(&self.repo, &target_path).map_err(|e| ResolveError::MissingId {
                kind: "scope_glob",
                id: e.to_string(),
            })?
        } else {
            // Explicit list: preserve deterministic order (sorted by id).
            let mut ms: Vec<ScopeMatch> = req
                .override_scopes
                .iter()
                .map(|id| {
                    let scope =
                        self.repo
                            .scopes
                            .get(id)
                            .ok_or_else(|| ResolveError::MissingId {
                                kind: "scopes",
                                id: id.clone(),
                            })?;

                    Ok(ScopeMatch {
                        id: id.clone(),
                        score: crate::resolv::scope_match::specificity_score(&scope.apply_to),
                        priority: scope.priority,
                    })
                })
                .collect::<Result<Vec<_>, ResolveError>>()?;

            ms.sort_by(|a, b| a.id.cmp(&b.id));
            ms
        };

        // Start from manifest defaults.
        let mut mode_id = self.repo.manifest.defaults.mode.clone();
        let mut policy_id = self.repo.manifest.defaults.policy.clone();
        let mut profile = self.repo.manifest.defaults.profile.clone();
        let mut backend = self
            .repo
            .manifest
            .defaults
            .backend
            .unwrap_or(BackendKind::VfsContainer);

        // Apply scopes from least specific to most specific so more specific wins.
        let mut scopes_for_apply = scopes_matched.clone();
        scopes_for_apply.sort_by(|a, b| {
            a.score
                .cmp(&b.score)
                .then_with(|| a.priority.cmp(&b.priority))
                .then_with(|| a.id.cmp(&b.id))
        });

        let mut enable_skills: BTreeSet<String> = BTreeSet::new();
        let mut disable_skills: BTreeSet<String> = BTreeSet::new();
        let mut include_snippets: BTreeSet<String> = BTreeSet::new();

        for m in &scopes_for_apply {
            let scope = self.repo.scopes.get(&m.id).unwrap();
            if let Some(mo) = &scope.overrides.mode {
                mode_id = mo.clone();
            }
            if let Some(po) = &scope.overrides.policy {
                policy_id = po.clone();
            }
            for s in &scope.overrides.enable_skills {
                enable_skills.insert(s.clone());
            }
            for s in &scope.overrides.disable_skills {
                disable_skills.insert(s.clone());
            }
            for snip in &scope.overrides.include_snippets {
                include_snippets.insert(snip.clone());
            }
        }

        // Apply state (if any) unless CLI overrides are provided.
        if let Some(state) = &self.repo.state {
            mode_id = state.mode.clone();
            if let Some(p) = &state.profile {
                profile = Some(p.clone());
            }
            if let Some(b) = state.backend {
                backend = b;
            }
        }

        // Apply CLI overrides.
        if let Some(mo) = &req.override_mode {
            mode_id = mo.clone();
        }
        if let Some(po) = &req.override_policy {
            policy_id = po.clone();
        }
        if let Some(pr) = &req.override_profile {
            profile = Some(pr.clone());
        }
        if let Some(b) = req.override_backend {
            backend = b;
        }

        // Validate resolved references exist.
        if !self.repo.modes.contains_key(&mode_id) {
            return Err(ResolveError::MissingId {
                kind: "modes",
                id: mode_id,
            });
        }
        if !self.repo.policies.contains_key(&policy_id) {
            return Err(ResolveError::MissingId {
                kind: "policies",
                id: policy_id,
            });
        }
        if let Some(pr) = &profile {
            if !self.repo.profiles.contains_key(pr) {
                return Err(ResolveError::MissingId {
                    kind: "profiles",
                    id: pr.clone(),
                });
            }
        }

        // Mode frontmatter contributes enable/disable skills + include snippets.
        if let Some(mode) = self.repo.modes.get(&mode_id) {
            if let Some(fm) = &mode.frontmatter {
                for s in &fm.enable_skills {
                    enable_skills.insert(s.clone());
                }
                for s in &fm.disable_skills {
                    disable_skills.insert(s.clone());
                }
                for snip in &fm.include_snippets {
                    include_snippets.insert(snip.clone());
                }

                if let Some(p) = &fm.policy {
                    policy_id = p.clone();
                }
            }
        }

        // Finalize skills: enabled minus disabled.
        let mut skill_ids_enabled: Vec<String> = enable_skills
            .into_iter()
            .filter(|s| !disable_skills.contains(s))
            .collect();
        skill_ids_enabled.sort();

        let mut snippet_ids_included: Vec<String> = include_snippets.into_iter().collect();
        snippet_ids_included.sort();

        // Validate referenced skills/snippets (strict for v1).
        for sid in &skill_ids_enabled {
            if !self.repo.skills.contains_key(sid) {
                return Err(ResolveError::MissingId {
                    kind: "skills",
                    id: sid.clone(),
                });
            }
        }
        for snip in &snippet_ids_included {
            if !self.repo.prompts.snippets.contains_key(snip) {
                return Err(ResolveError::MissingId {
                    kind: "snippets",
                    id: snip.clone(),
                });
            }
        }

        Ok(EffectiveConfig {
            mode_id,
            policy_id,
            profile,
            backend,
            scopes_matched,
            skill_ids_enabled,
            snippet_ids_included,
        })
    }
}

impl Default for ResolutionRequest {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::from("."),
            target_path: None,
            override_mode: None,
            override_policy: None,
            override_profile: None,
            override_backend: None,
            override_scopes: vec![],
            enable_user_overlay: false,
        }
    }
}
