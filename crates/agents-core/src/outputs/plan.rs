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

        let planned_out = build_planned_output(
            repo_root,
            agent_id,
            out,
            template_dir.clone(),
            render_ctx.clone(),
        )?;

        validate_renderer_sources(repo_root, repo, effective, &planned_out)?;

        planned.push(planned_out);
    }

    // Stable ordering by path then surface.
    planned.sort_by(|a, b| {
        a.path
            .as_str()
            .cmp(b.path.as_str())
            .then_with(|| a.surface.cmp(&b.surface))
    });

    // Collision handling.
    let planned = resolve_collisions(repo, agent_id, planned)?;

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

fn validate_renderer_sources(
    repo_root: &Path,
    repo: &RepoConfig,
    effective: &EffectiveConfig,
    out: &PlannedOutput,
) -> Result<(), PlanError> {
    let fail = |message: String| PlanError::InvalidRenderer {
        path: out.path.as_str().to_string(),
        message,
    };

    // Validate template existence for template renderer.
    if out.renderer.type_ == RendererType::Template {
        let template_name = out
            .renderer
            .template
            .as_deref()
            .unwrap_or("")
            .trim();

        if template_name.is_empty() {
            return Err(fail("template renderer requires `template`".to_string()));
        }

        let template_dir = out.template_dir.as_ref().ok_or_else(|| {
            fail("template renderer requires adapter template_dir".to_string())
        })?;

        if !template_exists(template_dir, template_name) {
            return Err(fail(format!(
                "unknown template source: {template_name}"
            )));
        }
    }

    // Validate each declared source for concat/copy/json_merge.
    if out.renderer.sources.is_empty() {
        return Ok(());
    }

    for raw in &out.renderer.sources {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(fail("renderer source must be non-empty".to_string()));
        }

        let (kind, val) = match raw.split_once(':') {
            Some((k, v)) => (Some(k), v),
            None => (None, raw),
        };

        match kind {
            Some("template") => {
                let template_dir = out
                    .template_dir
                    .as_ref()
                    .ok_or_else(|| fail("template source requires adapter template_dir".to_string()))?;
                let name = val.trim();
                if name.is_empty() {
                    return Err(fail("template:<name> must include a template name".to_string()));
                }
                if !template_exists(template_dir, name) {
                    return Err(fail(format!("unknown template source: {raw}")));
                }
            }
            Some("prompt") => {
                let p = val.trim();
                match p {
                    "base" | "project" | "composed" => {}
                    _ => return Err(fail(format!("unknown prompt source: {raw}"))),
                }
            }
            Some("snippet") => {
                let id = val.trim();
                if id.is_empty() {
                    return Err(fail("snippet:<id> must include a snippet id".to_string()));
                }
                if !effective.snippet_ids_included.iter().any(|x| x == id) {
                    return Err(fail(format!("snippet not included in effective config: {raw}")));
                }
                if !repo.prompts.snippets.contains_key(id) {
                    return Err(fail(format!("unknown snippet id: {raw}")));
                }
            }
            Some("repo") | Some("file") | None => {
                let rel = val.trim();
                if rel.is_empty() {
                    return Err(fail(format!("invalid file source: {raw}")));
                }

                let repo_rel = fsutil::repo_relpath_noexist(repo_root, Path::new(rel))
                    .map_err(|e| fail(format!("invalid file source: {raw}: {e}")))?;
                let abs = repo_root.join(repo_rel.as_str());
                if !abs.exists() {
                    return Err(fail(format!("missing file source: {raw}")));
                }
            }
            Some(other) => return Err(fail(format!("unknown renderer source kind: {other}"))),
        }
    }

    Ok(())
}

fn template_exists(template_dir: &Path, template_name: &str) -> bool {
    // Template names are stored as paths relative to the adapter templates directory.
    if template_name.is_empty() {
        return false;
    }

    // Reject absolute paths and parent traversal.
    let p = Path::new(template_name);
    if p.is_absolute() {
        return false;
    }
    for c in p.components() {
        if let std::path::Component::ParentDir = c {
            return false;
        }
    }

    template_dir.join(p).is_file()
}

fn build_planned_output(
    repo_root: &Path,
    _agent_id: &str,
    out: &AdapterOutput,
    template_dir: Option<std::path::PathBuf>,
    render_ctx: RenderContext,
) -> Result<PlannedOutput, PlanError> {
    let path = fsutil::repo_relpath_noexist(repo_root, Path::new(&out.path)).map_err(|e| {
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

fn resolve_collisions(
    repo: &RepoConfig,
    agent_id: &str,
    planned: Vec<PlannedOutput>,
) -> Result<Vec<PlannedOutput>, PlanError> {
    let shared_owner = repo
        .manifest
        .defaults
        .shared_surfaces_owner
        .clone()
        .unwrap_or_else(|| "core".to_string());

    // Enforce shared surface ownership even when there is no collision.
    for p in &planned {
        if p.collision != CollisionPolicy::SharedOwner {
            continue;
        }

        let surface = p.surface.as_deref().ok_or_else(|| PlanError::InvalidRenderer {
            path: p.path.as_str().to_string(),
            message: "collision=shared_owner requires a non-empty `surface`".to_string(),
        })?;

        if surface.is_empty() {
            return Err(PlanError::InvalidRenderer {
                path: p.path.as_str().to_string(),
                message: "collision=shared_owner requires a non-empty `surface`".to_string(),
            });
        }

        if shared_owner != agent_id {
            return Err(PlanError::SurfaceCollision {
                surface: surface.to_string(),
            });
        }
    }

    // Physical path collisions are always an error (multiple outputs writing the same file).
    let mut seen_paths: BTreeSet<String> = BTreeSet::new();
    for p in &planned {
        let key = p.path.as_str().to_string();
        if !seen_paths.insert(key.clone()) {
            return Err(PlanError::PathCollision { path: key });
        }
    }

    // Logical surface collisions.
    let mut by_surface: BTreeMap<String, Vec<PlannedOutput>> = BTreeMap::new();
    let mut without_surface: Vec<PlannedOutput> = vec![];
    for p in planned {
        if let Some(surface) = &p.surface {
            by_surface.entry(surface.clone()).or_default().push(p);
        } else {
            without_surface.push(p);
        }
    }

    let mut out: Vec<PlannedOutput> = vec![];
    out.extend(without_surface);

    for (surface, mut items) in by_surface {
        if items.len() == 1 {
            out.push(items.remove(0));
            continue;
        }

        // All colliding outputs must agree on collision policy.
        let policy = items[0].collision;
        if items.iter().any(|p| p.collision != policy) {
            return Err(PlanError::SurfaceCollision { surface });
        }

        match policy {
            CollisionPolicy::Error => return Err(PlanError::SurfaceCollision { surface }),
            CollisionPolicy::SharedOwner => {
                // Shared-owner surfaces may only be emitted once (by the designated owner).
                return Err(PlanError::SurfaceCollision { surface });
            }
            CollisionPolicy::Overwrite => {
                // Deterministic winner: lowest path.
                items.sort_by(|a, b| a.path.as_str().cmp(b.path.as_str()));
                out.push(items.remove(0));
            }
            CollisionPolicy::Merge => {
                // Deterministic merge order: ascending by path.
                items.sort_by(|a, b| a.path.as_str().cmp(b.path.as_str()));

                // Require compatible output settings.
                let first = &items[0];
                if items.iter().any(|p| p.format != first.format) {
                    return Err(PlanError::SurfaceCollision { surface });
                }
                if items
                    .iter()
                    .any(|p| !write_policy_eq(&p.write_policy, &first.write_policy))
                {
                    return Err(PlanError::SurfaceCollision { surface });
                }
                if items
                    .iter()
                    .any(|p| !drift_detection_eq(&p.drift_detection, &first.drift_detection))
                {
                    return Err(PlanError::SurfaceCollision { surface });
                }

                // Merge by creating a concat renderer over the original templates.
                let mut sources: Vec<String> = vec![];
                for p in &items {
                    if p.renderer.type_ != RendererType::Template {
                        return Err(PlanError::SurfaceCollision { surface });
                    }
                    let t = p.renderer.template.as_deref().unwrap_or("");
                    if t.is_empty() {
                        return Err(PlanError::SurfaceCollision { surface });
                    }
                    sources.push(format!("template:{t}"));
                }

                let mut merged = items.remove(0);
                merged.renderer.type_ = RendererType::Concat;
                merged.renderer.template = None;
                merged.renderer.sources = sources;
                // Keep the logical surface name.
                merged.surface = Some(surface.clone());
                out.push(merged);
            }
        }
    }

    // Ensure deterministic ordering.
    out.sort_by(|a, b| {
        a.path
            .as_str()
            .cmp(b.path.as_str())
            .then_with(|| a.surface.cmp(&b.surface))
    });

    Ok(out)
}

fn write_policy_eq(a: &WritePolicy, b: &WritePolicy) -> bool {
    a.mode == b.mode && a.gitignore == b.gitignore
}

fn drift_detection_eq(a: &DriftDetection, b: &DriftDetection) -> bool {
    a.method == b.method && a.stamp == b.stamp
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
