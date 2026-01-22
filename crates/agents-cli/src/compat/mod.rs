use std::fmt;
use std::path::Path;

use agents_core::loadag::LoaderOptions;
use agents_core::loadag::{load_repo_config, RepoConfig};
use agents_core::model::{Adapter, BackendKind};

use crate::{AppError, ErrorCategory, OutputMode};

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnforcementSummary {
    pub filesystem: String,
    pub network: String,
    pub exec: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompatEntry {
    pub agent_id: String,
    pub output_paths: Vec<String>,
    pub surfaces: Vec<String>,
    pub backend_preferred: BackendKind,
    pub backend_fallback: BackendKind,
    pub enforcement: EnforcementSummary,
    pub policy_mapping: String,
    pub limitations: Vec<String>,
}

#[derive(Debug)]
pub enum CompatError {
    MissingAdapter { agent_id: String },
}

impl fmt::Display for CompatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatError::MissingAdapter { agent_id } => {
                write!(f, "missing adapter: {agent_id}")
            }
        }
    }
}

impl std::error::Error for CompatError {}

#[derive(Debug, Clone)]
pub struct AdapterCompatSource {
    pub agent_id: String,
    pub output_paths: Vec<String>,
    pub surfaces: Vec<String>,
    pub backend_preferred: BackendKind,
    pub backend_fallback: BackendKind,
    pub has_capability_mapping: bool,
}

pub fn adapter_sources_from(adapter: &Adapter) -> AdapterCompatSource {
    let mut output_paths: Vec<String> = adapter.outputs.iter().map(|o| o.path.clone()).collect();
    output_paths.sort();

    let mut surfaces: Vec<String> = adapter
        .outputs
        .iter()
        .filter_map(|o| o.surface.clone())
        .collect();
    surfaces.sort();
    surfaces.dedup();

    AdapterCompatSource {
        agent_id: adapter.agent_id.clone(),
        output_paths,
        surfaces,
        backend_preferred: adapter.backend_defaults.preferred,
        backend_fallback: adapter.backend_defaults.fallback,
        has_capability_mapping: adapter.capability_mapping.is_some(),
    }
}

pub fn known_limitations(agent_id: &str) -> Vec<String> {
    match agent_id {
        "opencode" => vec!["requires opencode CLI installed".to_string()],
        "claude" => vec!["requires claude CLI installed".to_string()],
        "codex" => vec!["requires codex CLI installed".to_string()],
        "cursor" => vec!["requires Cursor to consume .cursor rules".to_string()],
        "copilot" => vec!["requires GitHub Copilot to read instructions".to_string()],
        "core" => vec!["shared surfaces only".to_string()],
        _ => vec![],
    }
}

pub fn build_matrix(repo: &RepoConfig) -> Result<Vec<CompatEntry>, CompatError> {
    let mut agent_ids = repo.manifest.enabled.adapters.clone();
    agent_ids.sort();

    let mut entries = Vec::new();
    for agent_id in agent_ids {
        let adapter = repo
            .adapters
            .get(&agent_id)
            .ok_or_else(|| CompatError::MissingAdapter {
                agent_id: agent_id.clone(),
            })?;

        let source = adapter_sources_from(adapter);
        let mut limitations = known_limitations(&source.agent_id);
        if let Some(lim) = backend_limitation(source.backend_preferred) {
            limitations.push(lim.to_string());
        }

        let policy_mapping = if source.has_capability_mapping {
            "custom (capabilityMapping)".to_string()
        } else {
            "advisory".to_string()
        };

        let enforcement = enforcement_for_backend(source.backend_preferred);

        entries.push(CompatEntry {
            agent_id: source.agent_id,
            output_paths: source.output_paths,
            surfaces: source.surfaces,
            backend_preferred: source.backend_preferred,
            backend_fallback: source.backend_fallback,
            enforcement,
            policy_mapping,
            limitations,
        });
    }

    Ok(entries)
}

pub fn cmd_compat(repo_root: &Path, output: OutputMode) -> Result<(), AppError> {
    let (repo, _report) = load_repo_config(
        repo_root,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    let _ = agents_core::schemas::validate_repo(repo_root);

    let entries = build_matrix(&repo).map_err(|e| AppError {
        category: ErrorCategory::Io,
        message: e.to_string(),
        context: vec![],
    })?;

    match output {
        OutputMode::Json => {
            let s = serde_json::to_string_pretty(&entries).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;
            println!("{s}");
        }
        OutputMode::Human => {
            print_compat_human(&entries);
        }
    }

    Ok(())
}

fn enforcement_for_backend(backend: BackendKind) -> EnforcementSummary {
    match backend {
        BackendKind::VfsContainer => EnforcementSummary {
            filesystem: "enforced via read-only mounts".to_string(),
            network: "best-effort (container networking)".to_string(),
            exec: "limited (advisory allow/deny)".to_string(),
        },
        BackendKind::Materialize => EnforcementSummary {
            filesystem: "not enforced (writes to repo)".to_string(),
            network: "advisory".to_string(),
            exec: "advisory".to_string(),
        },
        BackendKind::VfsMount => EnforcementSummary {
            filesystem: "copy-based workspace overlay".to_string(),
            network: "advisory".to_string(),
            exec: "advisory".to_string(),
        },
    }
}

fn backend_limitation(backend: BackendKind) -> Option<&'static str> {
    match backend {
        BackendKind::VfsContainer => Some("requires container runtime for vfs_container"),
        BackendKind::Materialize => Some("writes generated outputs into the repo"),
        BackendKind::VfsMount => Some("vfs_mount uses a temporary workspace copy"),
    }
}

fn print_compat_human(entries: &[CompatEntry]) {
    for (idx, entry) in entries.iter().enumerate() {
        if idx > 0 {
            println!();
        }

        println!("agent: {}", entry.agent_id);
        println!("outputs: {}", join_or_none(&entry.output_paths));
        println!("surfaces: {}", join_or_none(&entry.surfaces));
        println!(
            "backend: preferred {:?}, fallback {:?}",
            entry.backend_preferred, entry.backend_fallback
        );
        println!("policy_mapping: {}", entry.policy_mapping);
        println!(
            "enforcement: filesystem={}, network={}, exec={}",
            entry.enforcement.filesystem, entry.enforcement.network, entry.enforcement.exec
        );
        if entry.limitations.is_empty() {
            println!("limitations: <none>");
        } else {
            println!("limitations:");
            for lim in &entry.limitations {
                println!("- {lim}");
            }
        }
    }
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".to_string()
    } else {
        values.join(", ")
    }
}
