use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::fsutil;
use crate::loadag::RepoConfig;
use crate::model::{
    Adapter, AdapterOutput, CollisionPolicy, DriftDetection, DriftMethod, OutputFormat,
    RendererType, StampMethod, WriteMode, WritePolicy,
};
use crate::outputs::{OutputPlan, PlannedOutput, SourceMapSkeleton};
use crate::prompts::PromptComposer;
use crate::resolv::EffectiveConfig;
use crate::skillpl::SkillPlanner;
use crate::templ::{
    AdapterCtx, EffectiveCtx, EffectiveModeCtx, EffectiveSkillsCtx, GenerationCtx,
    GenerationStampCtx, RenderContext,
};

#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    #[error("unknown adapter: {agent_id}")]
    UnknownAdapter { agent_id: String },

    #[error("output collision at path: {path}")]
    PathCollision { path: String },

    #[error("surface collision: {surface}")]
    SurfaceCollision { surface: String },

    #[error("invalid renderer config for {path}: {message}")]
    InvalidRenderer { path: String, message: String },
}

#[derive(Debug)]
pub struct PlanResult {
    pub plan: OutputPlan,
    pub sources: Vec<SourceMapSkeleton>,
}

pub fn plan_outputs(
    repo_root: &Path,
    repo: RepoConfig,
    effective: &EffectiveConfig,
    agent_id: &str,
) -> Result<PlanResult, PlanError> {
    let adapter: Adapter =
        repo.adapters
            .get(agent_id)
            .cloned()
            .ok_or_else(|| PlanError::UnknownAdapter {
                agent_id: agent_id.to_string(),
            })?;

    let template_dir = repo.adapter_template_dirs.get(agent_id).cloned();

    let policy = repo
        .policies
        .get(&effective.policy_id)
        .cloned()
        .expect("effective policy exists (validated earlier)");

    let composer = PromptComposer::new(repo_root, repo.clone());
    let (prompts, _prompt_sources) =
        composer
            .compose(effective, &policy)
            .map_err(|e| PlanError::InvalidRenderer {
                path: "<prompts>".to_string(),
                message: e.to_string(),
            })?;

    let planner = SkillPlanner::new(repo.clone());
    let skills =
        planner
            .plan(effective, Some(agent_id))
            .map_err(|e| PlanError::InvalidRenderer {
                path: "<skills>".to_string(),
                message: e.to_string(),
            })?;

    let render_ctx = RenderContext {
        effective: EffectiveCtx {
            mode: {
                let mode = repo
                    .modes
                    .get(&effective.mode_id)
                    .expect("effective mode exists");
                EffectiveModeCtx {
                    frontmatter: mode.frontmatter.clone(),
                    body: mode.body.clone(),
                }
            },
            policy,
            skills: EffectiveSkillsCtx {
                ids: skills.enabled.iter().map(|s| s.id.clone()).collect(),
                summaries: vec![],
            },
            prompts,
        },
        profile: effective.profile.clone(),
        scopes_matched: effective
            .scopes_matched
            .iter()
            .map(|s| s.id.clone())
            .collect(),
        generation: GenerationCtx {
            stamp: GenerationStampCtx {
                generator: "agents".to_string(),
                adapter_agent_id: agent_id.to_string(),
                mode: effective.mode_id.clone(),
                profile: effective.profile.clone(),
            },
        },
        adapter: AdapterCtx {
            agent_id: agent_id.to_string(),
        },
        x: None,
    };

    let outputs = evaluate_outputs(
        repo_root,
        &repo,
        effective,
        agent_id,
        &adapter,
        template_dir.clone(),
        &render_ctx,
    )?;

    let sources = build_source_map_skeletons(effective, agent_id, &outputs);

    Ok(PlanResult {
        plan: OutputPlan {
            agent_id: agent_id.to_string(),
            backend: effective.backend,
            outputs,
        },
        sources,
    })
}

fn evaluate_outputs(
    repo_root: &Path,
    repo: &RepoConfig,
    effective: &EffectiveConfig,
    agent_id: &str,
    adapter: &Adapter,
    template_dir: Option<std::path::PathBuf>,
    render_ctx: &RenderContext,
) -> Result<Vec<PlannedOutput>, PlanError> {
    let mut planned: Vec<PlannedOutput> = vec![];

    for out in &adapter.outputs {
        if !condition_allows(out, effective) {
            continue;
        }

        validate_renderer(out)?;

        planned.push(build_planned_output(
            repo_root,
            agent_id,
            out,
            template_dir.clone(),
            render_ctx.clone(),
        )?);
    }

    // Stable ordering by path then surface.
    planned.sort_by(|a, b| {
        a.path
            .as_str()
            .cmp(b.path.as_str())
            .then_with(|| a.surface.cmp(&b.surface))
    });

    // Collision detection.
    detect_collisions(repo, &planned)?;

    Ok(planned)
}

fn condition_allows(out: &AdapterOutput, effective: &EffectiveConfig) -> bool {
    if let Some(cond) = &out.condition {
        if !cond.backend_in.is_empty() && !cond.backend_in.iter().any(|b| *b == effective.backend) {
            return false;
        }

        if !cond.profile_in.is_empty() {
            match &effective.profile {
                Some(p) if cond.profile_in.iter().any(|x| x == p) => {}
                _ => return false,
            }
        }
    }

    true
}

fn validate_renderer(out: &AdapterOutput) -> Result<(), PlanError> {
    match out.renderer.type_ {
        RendererType::Template => {
            if out.renderer.template.as_deref().unwrap_or("").is_empty() {
                return Err(PlanError::InvalidRenderer {
                    path: out.path.clone(),
                    message: "template renderer requires `template`".to_string(),
                });
            }
        }
        RendererType::Concat => {
            if out.renderer.sources.is_empty() {
                return Err(PlanError::InvalidRenderer {
                    path: out.path.clone(),
                    message: "concat renderer requires `sources`".to_string(),
                });
            }
        }
        RendererType::Copy => {
            if out.renderer.sources.is_empty() {
                return Err(PlanError::InvalidRenderer {
                    path: out.path.clone(),
                    message: "copy renderer requires `sources`".to_string(),
                });
            }
        }
        RendererType::JsonMerge => {
            if out.renderer.sources.is_empty() {
                return Err(PlanError::InvalidRenderer {
                    path: out.path.clone(),
                    message: "json_merge renderer requires `sources`".to_string(),
                });
            }
            if out.renderer.json_merge_strategy.is_none() {
                return Err(PlanError::InvalidRenderer {
                    path: out.path.clone(),
                    message: "json_merge renderer requires `jsonMergeStrategy`".to_string(),
                });
            }
        }
    }

    Ok(())
}

fn build_planned_output(
    repo_root: &Path,
    _agent_id: &str,
    out: &AdapterOutput,
    template_dir: Option<std::path::PathBuf>,
    render_ctx: RenderContext,
) -> Result<PlannedOutput, PlanError> {
    let path = fsutil::repo_relpath(repo_root, Path::new(&out.path)).map_err(|e| {
        PlanError::InvalidRenderer {
            path: out.path.clone(),
            message: e.to_string(),
        }
    })?;

    let format = out.format.unwrap_or(OutputFormat::Text);
    let collision = out.collision.unwrap_or(CollisionPolicy::Error);

    let write_policy = out.write_policy.clone().unwrap_or(WritePolicy {
        mode: Some(WriteMode::IfGenerated),
        gitignore: false,
    });

    let drift_detection = out.drift_detection.clone().unwrap_or(DriftDetection {
        method: Some(DriftMethod::Sha256),
        stamp: Some(StampMethod::Comment),
    });

    Ok(PlannedOutput {
        path,
        format,
        surface: out.surface.clone(),
        collision,
        renderer: out.renderer.clone(),
        write_policy,
        drift_detection,
        template_dir,
        render_context: render_ctx,
    })
}

fn detect_collisions(repo: &RepoConfig, planned: &[PlannedOutput]) -> Result<(), PlanError> {
    // Physical path collisions.
    let mut seen_paths: BTreeSet<String> = BTreeSet::new();
    for p in planned {
        let key = p.path.as_str().to_string();
        if !seen_paths.insert(key.clone()) {
            return Err(PlanError::PathCollision { path: key });
        }
    }

    // Logical surface collisions.
    let mut by_surface: BTreeMap<String, Vec<&PlannedOutput>> = BTreeMap::new();
    for p in planned {
        if let Some(surface) = &p.surface {
            by_surface.entry(surface.clone()).or_default().push(p);
        }
    }

    for (surface, items) in by_surface {
        if items.len() <= 1 {
            continue;
        }

        // If any of the colliding items are shared_owner, enforce manifest owner.
        let any_shared_owner = items
            .iter()
            .any(|p| p.collision == CollisionPolicy::SharedOwner);
        if any_shared_owner {
            let owner = repo
                .manifest
                .defaults
                .shared_surfaces_owner
                .clone()
                .unwrap_or_else(|| "core".to_string());

            // Require that only the owner adapter defines this surface.
            if owner != "core" {
                // v1: we only support core owner; other ownership is deferred.
                return Err(PlanError::SurfaceCollision { surface });
            }

            return Err(PlanError::SurfaceCollision { surface });
        }

        // For now, any other collision is an error unless explicitly merge/overwrite.
        // Full merge/overwrite is implemented in a later step.
        return Err(PlanError::SurfaceCollision { surface });
    }

    Ok(())
}

fn build_source_map_skeletons(
    effective: &EffectiveConfig,
    agent_id: &str,
    planned: &[PlannedOutput],
) -> Vec<SourceMapSkeleton> {
    planned
        .iter()
        .map(|p| SourceMapSkeleton {
            adapter_id: agent_id.to_string(),
            output_path: p.path.as_str().to_string(),
            template: p.renderer.template.clone(),
            mode_id: effective.mode_id.clone(),
            policy_id: effective.policy_id.clone(),
            skill_ids: effective.skill_ids_enabled.clone(),
            snippet_ids: effective.snippet_ids_included.clone(),
        })
        .collect()
}
