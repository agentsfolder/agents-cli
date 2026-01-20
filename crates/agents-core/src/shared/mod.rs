use crate::model::{
    Adapter, AdapterOutput, BackendDefaults, BackendKind, CollisionPolicy, DriftDetection,
    DriftMethod, OutputFormat, OutputRenderer, RendererType, StampMethod, WriteMode, WritePolicy,
};

pub const CORE_ADAPTER_ID: &str = "core";
pub const AGENTS_MD_PATH: &str = "AGENTS.md";
pub const AGENTS_MD_SURFACE: &str = "shared:AGENTS.md";
pub const AGENTS_MD_TEMPLATE: &str = "AGENTS.md.hbs";

pub fn builtin_template(agent_id: &str, template_name: &str) -> Option<&'static str> {
    if agent_id == CORE_ADAPTER_ID && template_name == AGENTS_MD_TEMPLATE {
        return Some(include_str!("AGENTS.md.hbs"));
    }
    None
}

pub fn inject_builtin_adapters(adapters: &mut std::collections::BTreeMap<String, Adapter>) {
    if adapters.contains_key(CORE_ADAPTER_ID) {
        return;
    }

    adapters.insert(CORE_ADAPTER_ID.to_string(), builtin_core_adapter());
}

fn builtin_core_adapter() -> Adapter {
    Adapter {
        agent_id: CORE_ADAPTER_ID.to_string(),
        version: "0.1".to_string(),
        backend_defaults: BackendDefaults {
            preferred: BackendKind::Materialize,
            fallback: BackendKind::Materialize,
        },
        capability_mapping: None,
        outputs: vec![AdapterOutput {
            path: AGENTS_MD_PATH.to_string(),
            format: Some(OutputFormat::Md),
            surface: Some(AGENTS_MD_SURFACE.to_string()),
            collision: Some(CollisionPolicy::SharedOwner),
            condition: None,
            renderer: OutputRenderer {
                type_: RendererType::Template,
                template: Some(AGENTS_MD_TEMPLATE.to_string()),
                sources: vec![],
                json_merge_strategy: None,
            },
            write_policy: Some(WritePolicy {
                mode: Some(WriteMode::IfGenerated),
                gitignore: false,
            }),
            drift_detection: Some(DriftDetection {
                method: Some(DriftMethod::Sha256),
                stamp: Some(StampMethod::Comment),
            }),
        }],
        tests: None,
        x: None,
    }
}
