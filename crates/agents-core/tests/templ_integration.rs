use std::path::Path;

use agents_core::templ::{RenderContext, TemplateEngine};

fn minimal_ctx() -> RenderContext {
    agents_core::templ::RenderContext {
        effective: agents_core::templ::EffectiveCtx {
            mode: agents_core::templ::EffectiveModeCtx {
                frontmatter: None,
                body: "mode".to_string(),
            },
            policy: agents_core::model::Policy {
                id: "p".to_string(),
                description: "d".to_string(),
                capabilities: agents_core::model::Capabilities {
                    filesystem: None,
                    exec: None,
                    network: None,
                    mcp: None,
                },
                paths: agents_core::model::Paths {
                    allow: vec![],
                    deny: vec![],
                    redact: vec![],
                },
                confirmations: agents_core::model::Confirmations {
                    required_for: vec![],
                },
                limits: None,
                x: None,
            },
            skills: agents_core::templ::EffectiveSkillsCtx {
                ids: vec![],
                summaries: vec![],
            },
            prompts: agents_core::prompts::EffectivePrompts {
                base_md: "Base".to_string(),
                project_md: "Project".to_string(),
                snippets: vec![],
                composed_md: "Base\n\nProject\n".to_string(),
            },
        },
        backend: agents_core::model::BackendKind::VfsContainer,
        profile: None,
        scopes_matched: vec![],
        scope: None,
        generation: agents_core::templ::GenerationCtx {
            stamp: agents_core::templ::GenerationStampCtx {
                generator: "agents".to_string(),
                adapter_agent_id: "x".to_string(),
                mode: "default".to_string(),
                profile: None,
            },
        },
        adapter: agents_core::templ::AdapterCtx {
            agent_id: "codex".to_string(),
        },
        x: None,
    }
}

#[test]
fn render_template_from_dir_works() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/templ/minimal_adapter/templates");

    let mut engine = TemplateEngine::new();
    engine.register_partials_from_dir(&dir).unwrap();

    let out = engine.render("hello.txt.hbs", &minimal_ctx()).unwrap();
    assert_eq!(out, "Hello codex\n");
}
