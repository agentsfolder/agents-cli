use agents_core::model::BackendKind;

#[derive(Debug, Clone)]
pub struct AgentSpec {
    pub id: &'static str,

    /// Executable (or entrypoint) to run.
    pub exec: &'static str,

    /// Default backend preference for `agents run` when the repo does not specify one.
    pub preferred_backend: BackendKind,
}

pub fn default_agent_registry() -> Vec<AgentSpec> {
    vec![
        AgentSpec {
            id: "opencode",
            exec: "opencode",
            preferred_backend: BackendKind::VfsContainer,
        },
        AgentSpec {
            id: "claude",
            exec: "claude",
            preferred_backend: BackendKind::VfsContainer,
        },
        AgentSpec {
            id: "codex",
            exec: "codex",
            preferred_backend: BackendKind::VfsContainer,
        },
    ]
}

pub fn lookup_agent_spec<'a>(registry: &'a [AgentSpec], id: &str) -> Option<&'a AgentSpec> {
    registry.iter().find(|a| a.id == id)
}
