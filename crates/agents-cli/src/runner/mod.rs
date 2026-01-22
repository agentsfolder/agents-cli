use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use agents_core::fsutil;
use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};
use agents_core::matwiz::{Backend as MatwizBackend, MaterializeBackend};
use agents_core::model::BackendKind;
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::stamps::{classify as classify_drift, parse_stamp};
use agents_core::vfsmnt::{OverlayFile, VfsMountOptions};

use crate::{AppError, ErrorCategory};

pub mod registry;

#[derive(Debug, Clone)]
pub struct RunOptions {
    /// The command to execute (also used as adapter id by default).
    pub agent_cmd: String,

    /// Adapter id to use for output planning/rendering.
    pub adapter: Option<String>,

    pub backend: Option<agents_core::model::BackendKind>,
    pub mode: Option<String>,
    pub profile: Option<String>,

    /// Args after `--`.
    pub passthrough: Vec<String>,

    pub verbose: bool,
}

pub fn cmd_run(repo_root: &Path, opts: RunOptions) -> Result<(), AppError> {
    let registry = registry::default_agent_registry();
    let agent_spec = registry::lookup_agent_spec(&registry, &opts.agent_cmd).cloned();
    let agent_cmd = agent_spec
        .as_ref()
        .map(|spec| spec.exec.to_string())
        .unwrap_or_else(|| opts.agent_cmd.clone());
    let adapter_id = opts.adapter.clone().unwrap_or_else(|| {
        agent_spec
            .as_ref()
            .map(|spec| spec.id.to_string())
            .unwrap_or_else(|| opts.agent_cmd.clone())
    });

    let (repo, _report) = load_repo_config(
        repo_root,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .map_err(|e| match e {
        LoadError::NotInitialized { .. } => AppError::not_initialized(repo_root),
        other => AppError {
            category: ErrorCategory::Io,
            message: other.to_string(),
            context: vec![],
        },
    })?;

    // Validate schemas if present.
    if repo_root.join(".agents/schemas").is_dir() {
        agents_core::schemas::validate_repo(repo_root).map_err(|err| AppError {
            category: ErrorCategory::SchemaInvalid,
            message: format!("schema invalid: {} ({})", err.path.display(), err.schema),
            context: {
                let mut c = vec![format!("pointer: {}", err.pointer), err.message];
                if let Some(h) = err.hint {
                    c.push(h);
                }
                c
            },
        })?;
    }

    let state_backend = repo.state.as_ref().and_then(|s| s.backend);
    let backend_override = if opts.backend.is_none()
        && repo.manifest.defaults.backend.is_none()
        && state_backend.is_none()
    {
        agent_spec.as_ref().map(|spec| spec.preferred_backend)
    } else {
        None
    };
    let resolver = Resolver::new(repo.clone());
    let req = ResolutionRequest {
        repo_root: repo_root.to_path_buf(),
        override_mode: opts.mode.clone(),
        override_profile: opts.profile.clone(),
        override_backend: opts.backend.or(backend_override),
        ..Default::default()
    };
    let effective = resolver.resolve(&req).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    warn_policy_risks(&repo, &effective);

    let plan_res =
        plan_outputs(repo_root, repo.clone(), &effective, &adapter_id).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![],
        })?;

    // Persist explain maps so `agents explain` works for generated outputs.
    crate::explnx::persist_source_maps(repo_root, &plan_res)?;

    // Render all planned outputs once.
    let mut rendered: Vec<RenderedItem> = vec![];
    for planned in &plan_res.plan.outputs {
        let r = render_planned_output(repo_root, planned).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", planned.path.as_str())],
        })?;

        rendered.push(RenderedItem {
            path: planned.path.as_str().to_string(),
            content_without_stamp: r.content_without_stamp,
            content_with_stamp: r.content_with_stamp,
        });
    }

    rendered.sort_by(|a, b| a.path.cmp(&b.path));

    if opts.verbose {
        eprintln!(
            "run: agent={} adapter={} backend={:?}",
            agent_cmd, adapter_id, effective.backend
        );
        if let Some(spec) = &agent_spec {
            eprintln!(
                "run: registry id={} exec={} preferred_backend={:?}",
                spec.id, spec.exec, spec.preferred_backend
            );
        }
        for p in &rendered {
            eprintln!("run: output: {}", p.path);
        }
    }

    match effective.backend {
        BackendKind::Materialize => {
            apply_materialize(repo_root, &plan_res.plan.outputs, &rendered)?;

            let status = run_host_agent(repo_root, &agent_cmd, &opts.passthrough)?;
            exit_with_status(status)
        }

        BackendKind::VfsContainer => {
            let tmp = fsutil::temp_generation_dir("agents-run").map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;
            let outputs_dir: PathBuf = tmp.path().to_path_buf();

            for item in &rendered {
                let dest = outputs_dir.join(&item.path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| AppError {
                        category: ErrorCategory::Io,
                        message: e.to_string(),
                        context: vec![format!("path: {}", parent.display())],
                    })?;
                }
                fsutil::atomic_write(&dest, item.content_with_stamp.as_bytes()).map_err(|e| {
                    AppError {
                        category: ErrorCategory::Io,
                        message: e.to_string(),
                        context: vec![format!("path: {}", dest.display())],
                    }
                })?;
            }

            let policy = repo
                .policies
                .get(&effective.policy_id)
                .ok_or_else(|| AppError {
                    category: ErrorCategory::Io,
                    message: "missing effective policy".to_string(),
                    context: vec![format!("policy: {}", effective.policy_id)],
                })?;

            let network_enabled = policy
                .capabilities
                .network
                .as_ref()
                .map(|n| n.enabled)
                .unwrap_or(false);
            let fs_write_enabled = policy
                .capabilities
                .filesystem
                .as_ref()
                .map(|f| f.write)
                .unwrap_or(true);

            let cmd = build_agent_cmd(&agent_cmd, &opts.passthrough);

            let inv = agents_core::vfsctr::run::VfsContainerInvocation {
                repo_root: repo_root.to_path_buf(),
                outputs_dir,
                image: agents_core::vfsctr::run::default_image(),
                cmd,
                env: BTreeMap::new(),
                verbose: opts.verbose,
                deny_network: !network_enabled,
                deny_writes: !fs_write_enabled,
            };

            let docker = agents_core::vfsctr::docker::DockerRuntime::new();
            let status = inv.run_interactive(&docker).map_err(|e| AppError {
                category: ErrorCategory::ExternalToolMissing,
                message: e.to_string(),
                context: vec!["hint: ensure docker is installed and running".to_string()],
            })?;

            // Keep temp dir alive until container exits.
            let _tmp = tmp;
            exit_with_status(status)
        }

        BackendKind::VfsMount => {
            let policy = repo
                .policies
                .get(&effective.policy_id)
                .ok_or_else(|| AppError {
                    category: ErrorCategory::Io,
                    message: "missing effective policy".to_string(),
                    context: vec![format!("policy: {}", effective.policy_id)],
                })?;

            let fs_write_enabled = policy
                .capabilities
                .filesystem
                .as_ref()
                .map(|f| f.write)
                .unwrap_or(true);

            let overlays: Vec<OverlayFile> = rendered
                .iter()
                .map(|item| OverlayFile {
                    rel_path: item.path.clone(),
                    bytes: item.content_with_stamp.as_bytes().to_vec(),
                })
                .collect();

            let workspace = agents_core::vfsmnt::create_workspace(
                repo_root,
                &overlays,
                &VfsMountOptions {
                    deny_writes: !fs_write_enabled,
                    verbose: opts.verbose,
                },
            )
            .map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;

            println!("mount: {}", workspace.path().display());

            let status = run_host_agent(workspace.path(), &agent_cmd, &opts.passthrough)?;
            exit_with_status(status)
        }
    }
}

#[derive(Debug, Clone)]
struct RenderedItem {
    path: String,
    content_without_stamp: String,
    content_with_stamp: String,
}

fn warn_policy_risks(
    repo: &agents_core::loadag::RepoConfig,
    effective: &agents_core::resolv::EffectiveConfig,
) {
    let Some(policy) = repo.policies.get(&effective.policy_id) else {
        return;
    };

    let network_enabled = policy
        .capabilities
        .network
        .as_ref()
        .map(|n| n.enabled)
        .unwrap_or(false);
    let exec = policy.capabilities.exec.as_ref();
    let unrestricted_exec = exec
        .map(|e| e.enabled && e.allow.is_empty() && e.deny.is_empty())
        .unwrap_or(false);

    if network_enabled || unrestricted_exec {
        eprintln!("warning: policy may allow risky capabilities");
        if network_enabled {
            eprintln!("warning: network enabled");
        }
        if unrestricted_exec {
            eprintln!("warning: exec enabled without allow/deny lists");
        }
    }
}

fn apply_materialize(
    repo_root: &Path,
    planned: &[agents_core::outputs::PlannedOutput],
    rendered: &[RenderedItem],
) -> Result<(), AppError> {
    let backend = MaterializeBackend;

    let plan = agents_core::outputs::OutputPlan {
        agent_id: "<run>".to_string(),
        backend: BackendKind::Materialize,
        outputs: planned.to_vec(),
    };

    let mut session = backend.prepare(repo_root, &plan).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let mut outs: Vec<agents_core::matwiz::RenderedOutput> = vec![];
    for p in planned {
        let item = rendered
            .iter()
            .find(|x| x.path == p.path.as_str())
            .expect("rendered output present");

        let dest = repo_root.join(p.path.as_str());
        let drift_status = classify_drift(&dest, &item.content_without_stamp, &p.drift_detection)
            .map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", dest.display())],
        })?;

        let stamp = parse_stamp(&item.content_with_stamp).ok_or_else(|| AppError {
            category: ErrorCategory::Io,
            message: "rendered output missing stamp".to_string(),
            context: vec![format!("path: {}", p.path.as_str())],
        })?;

        outs.push(agents_core::matwiz::RenderedOutput {
            path: p.path.clone(),
            bytes: item.content_with_stamp.as_bytes().to_vec(),
            stamp_meta: stamp.meta,
            drift_status,
        });
    }

    let report = backend.apply(&mut session, &outs).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    if !report.conflicts.is_empty() {
        return Err(AppError {
            category: ErrorCategory::Conflict,
            message: "conflicts detected while preparing outputs".to_string(),
            context: report
                .conflicts
                .iter()
                .map(|p| format!("path: {}", p.as_str()))
                .collect(),
        });
    }

    Ok(())
}

fn run_host_agent(
    repo_root: &Path,
    exec: &str,
    passthrough: &[String],
) -> Result<std::process::ExitStatus, AppError> {
    let cmd = build_agent_cmd(exec, passthrough);
    let (exec, args) = cmd.split_first().expect("agent command present");

    let status = std::process::Command::new(exec)
        .args(args)
        .current_dir(repo_root)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| AppError {
            category: ErrorCategory::ExternalToolMissing,
            message: e.to_string(),
            context: vec![format!("exec: {exec}")],
        })?;

    Ok(status)
}

fn build_agent_cmd(agent_cmd: &str, passthrough: &[String]) -> Vec<String> {
    let mut cmd = Vec::with_capacity(1 + passthrough.len());
    cmd.push(agent_cmd.to_string());
    cmd.extend(passthrough.iter().cloned());
    cmd
}

fn exit_with_status(status: std::process::ExitStatus) -> Result<(), AppError> {
    let code = status.code().unwrap_or(1);
    if code == 0 {
        Ok(())
    } else {
        Err(AppError {
            category: ErrorCategory::AgentExit { code },
            message: "".to_string(),
            context: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::build_agent_cmd;

    #[test]
    fn build_agent_cmd_appends_passthrough_args() {
        let cmd = build_agent_cmd("opencode", &["--help".to_string(), "--json".to_string()]);

        assert_eq!(cmd, vec!["opencode", "--help", "--json"]);
    }

    #[test]
    fn build_agent_cmd_handles_empty_passthrough() {
        let cmd = build_agent_cmd("claude", &[]);

        assert_eq!(cmd, vec!["claude"]);
    }
}
