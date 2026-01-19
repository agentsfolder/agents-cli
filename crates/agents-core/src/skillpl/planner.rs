use std::collections::BTreeSet;

use crate::loadag::RepoConfig;
use crate::model::{BackendKind, Skill};
use crate::resolv::{EffectiveConfig, ScopeMatch};
use crate::skillpl::{EffectiveSkills, SkillRef};

#[derive(Debug, thiserror::Error)]
pub enum SkillPlanError {
    #[error("missing skill id: {id}")]
    MissingSkill { id: String },

    #[error("skill not enabled in manifest: {id}")]
    SkillNotEnabled { id: String },

    #[error("skill incompatible with agent {agent_id}: {skill_id}")]
    IncompatibleAgent { agent_id: String, skill_id: String },

    #[error("skill incompatible with backend {backend:?}: {skill_id}")]
    IncompatibleBackend {
        backend: BackendKind,
        skill_id: String,
    },
}

#[derive(Debug)]
pub struct SkillPlanner {
    repo: RepoConfig,
}

impl SkillPlanner {
    pub fn new(repo: RepoConfig) -> Self {
        Self { repo }
    }

    pub fn plan(
        &self,
        effective: &EffectiveConfig,
        agent_id: Option<&str>,
    ) -> Result<EffectiveSkills, SkillPlanError> {
        let mut candidate: BTreeSet<String> = BTreeSet::new();

        // Base set: manifest enabled skills.
        for id in &self.repo.manifest.enabled.skills {
            candidate.insert(id.clone());
        }

        // Scope-specific enable/disable must be applied in deterministic order.
        // We re-derive scope apply order from the match list.
        let mut scopes_for_apply: Vec<ScopeMatch> = effective.scopes_matched.clone();
        scopes_for_apply.sort_by(|a, b| {
            a.score
                .cmp(&b.score)
                .then_with(|| a.priority.cmp(&b.priority))
                .then_with(|| a.id.cmp(&b.id))
        });

        for sm in scopes_for_apply {
            if let Some(scope) = self.repo.scopes.get(&sm.id) {
                for s in &scope.overrides.enable_skills {
                    candidate.insert(s.clone());
                }
                for s in &scope.overrides.disable_skills {
                    candidate.remove(s);
                }
            }
        }

        // Mode frontmatter enable/disable.
        if let Some(mode) = self.repo.modes.get(&effective.mode_id) {
            if let Some(fm) = &mode.frontmatter {
                for s in &fm.enable_skills {
                    candidate.insert(s.clone());
                }
                for s in &fm.disable_skills {
                    candidate.remove(s);
                }
            }
        }

        // Convert to stable Vec.
        let skill_ids: Vec<String> = candidate.into_iter().collect();

        let mut enabled: Vec<SkillRef> = Vec::new();
        for id in &skill_ids {
            // Enforced: skill must be listed in manifest enabled set.
            if !self.repo.manifest.enabled.skills.contains(id) {
                return Err(SkillPlanError::SkillNotEnabled { id: id.clone() });
            }

            let skill: Skill = self
                .repo
                .skills
                .get(id)
                .cloned()
                .ok_or_else(|| SkillPlanError::MissingSkill { id: id.clone() })?;

            let dir = self
                .repo
                .skill_dirs
                .get(id)
                .cloned()
                .unwrap_or_else(|| self.repo.repo_root.join(".agents/skills").join(id));

            // Compatibility
            if let Some(comp) = &skill.compatibility {
                if let Some(agent_id) = agent_id {
                    if !comp.agents.is_empty() && !comp.agents.iter().any(|a| a == agent_id) {
                        return Err(SkillPlanError::IncompatibleAgent {
                            agent_id: agent_id.to_string(),
                            skill_id: id.clone(),
                        });
                    }
                }

                if !comp.backends.is_empty()
                    && !comp.backends.iter().any(|b| *b == effective.backend)
                {
                    return Err(SkillPlanError::IncompatibleBackend {
                        backend: effective.backend,
                        skill_id: id.clone(),
                    });
                }
            }

            enabled.push(SkillRef {
                id: id.clone(),
                dir,
                skill,
            });
        }

        // Already stable-sorted by BTreeSet collection.
        Ok(EffectiveSkills {
            enabled,
            disabled: vec![],
            warnings: vec![],
            backend: effective.backend,
            agent_id: agent_id.map(|s| s.to_string()),
        })
    }
}
