use std::fs;

use agents_core::fsutil;
use agents_core::matwiz::{Backend, ConflictReason, MaterializeBackend, RenderedOutput};
use agents_core::model::{
    CollisionPolicy, DriftDetection, DriftMethod, JsonMergeStrategy, OutputFormat, OutputRenderer,
    RendererType, StampMethod, WriteMode, WritePolicy,
};
use agents_core::outputs::{OutputPlan, PlannedOutput};
use agents_core::stamps::{apply_stamp, classify, compute_sha256_hex, DriftStatus, StampMeta};
use agents_core::templ::{
    AdapterCtx, EffectiveCtx, EffectiveModeCtx, EffectiveSkillsCtx, GenerationCtx,
    GenerationStampCtx, RenderContext,
};

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn dummy_render_context() -> RenderContext {
    RenderContext {
        effective: EffectiveCtx {
            mode: EffectiveModeCtx {
                frontmatter: None,
                body: "".to_string(),
            },
            policy: agents_core::model::Policy {
                id: "safe".to_string(),
                description: "".to_string(),
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
            skills: EffectiveSkillsCtx {
                ids: vec![],
                summaries: vec![],
            },
            prompts: agents_core::prompts::EffectivePrompts {
                base_md: "".to_string(),
                project_md: "".to_string(),
                snippets: vec![],
                composed_md: "".to_string(),
            },
        },
        profile: None,
        scopes_matched: vec![],
        generation: GenerationCtx {
            stamp: GenerationStampCtx {
                generator: "agents".to_string(),
                adapter_agent_id: "a".to_string(),
                mode: "default".to_string(),
                profile: None,
            },
        },
        adapter: AdapterCtx {
            agent_id: "a".to_string(),
        },
        x: None,
    }
}

fn repo_path_for_existing(repo_root: &std::path::Path, rel: &str) -> fsutil::RepoPath {
    let abs = repo_root.join(rel);
    write_file(&abs, "");
    fsutil::repo_relpath(repo_root, &abs).unwrap()
}

fn planned_output(path: fsutil::RepoPath, write_mode: WriteMode, gitignore: bool) -> PlannedOutput {
    PlannedOutput {
        path,
        format: OutputFormat::Text,
        surface: None,
        collision: CollisionPolicy::Error,
        renderer: OutputRenderer {
            type_: RendererType::Template,
            template: Some("t.hbs".to_string()),
            sources: vec![],
            json_merge_strategy: Some(JsonMergeStrategy::Deep),
        },
        write_policy: WritePolicy {
            mode: Some(write_mode),
            gitignore,
        },
        drift_detection: DriftDetection {
            method: Some(DriftMethod::Sha256),
            stamp: Some(StampMethod::Comment),
        },
        template_dir: None,
        render_context: dummy_render_context(),
    }
}

fn stamp_meta(profile: Option<&str>, content_without_stamp: &str) -> StampMeta {
    StampMeta {
        generator: "agents".to_string(),
        adapter_agent_id: "a".to_string(),
        manifest_spec_version: "0.1".to_string(),
        mode: "default".to_string(),
        policy: "safe".to_string(),
        backend: agents_core::model::manifest::BackendKind::Materialize,
        profile: profile.map(|s| s.to_string()),
        content_sha256: compute_sha256_hex(content_without_stamp),
    }
}

fn rendered(path: fsutil::RepoPath, meta: StampMeta, drift_status: DriftStatus, content: &str) -> RenderedOutput {
    let stamped = apply_stamp(content, &meta, StampMethod::Comment).unwrap();
    RenderedOutput {
        path,
        bytes: stamped.into_bytes(),
        stamp_meta: meta,
        drift_status,
    }
}

#[test]
fn write_new_file() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();

    let path = repo_path_for_existing(repo_root, "out.txt");
    fs::remove_file(repo_root.join("out.txt")).unwrap();

    let plan = OutputPlan {
        agent_id: "a".to_string(),
        backend: agents_core::model::manifest::BackendKind::Materialize,
        outputs: vec![planned_output(path.clone(), WriteMode::IfGenerated, false)],
    };

    let drift = classify(
        &repo_root.join("out.txt"),
        "hello\n",
        &DriftDetection {
            method: Some(DriftMethod::Sha256),
            stamp: Some(StampMethod::Comment),
        },
    )
    .unwrap();
    assert_eq!(drift, DriftStatus::Missing);

    let meta = stamp_meta(None, "hello\n");
    let out = rendered(path.clone(), meta, drift, "hello\n");

    let backend = MaterializeBackend;
    let mut session = backend.prepare(repo_root, &plan).unwrap();
    let report = backend.apply(&mut session, &[out]).unwrap();

    assert_eq!(report.written, vec![path]);
    assert_eq!(fs::read_to_string(repo_root.join("out.txt")).unwrap().contains("hello"), true);
}

#[test]
fn overwrite_stamped_file_with_if_generated() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();

    let path = repo_path_for_existing(repo_root, "out.txt");

    // Existing file: same body, but different stamp meta (profile differs).
    let old = stamp_meta(Some("old"), "hello\n");
    let existing = apply_stamp("hello\n", &old, StampMethod::Comment).unwrap();
    write_file(&repo_root.join("out.txt"), &existing);

    let plan = OutputPlan {
        agent_id: "a".to_string(),
        backend: agents_core::model::manifest::BackendKind::Materialize,
        outputs: vec![planned_output(path.clone(), WriteMode::IfGenerated, false)],
    };

    let drift = classify(
        &repo_root.join("out.txt"),
        "hello\n",
        &DriftDetection {
            method: Some(DriftMethod::Sha256),
            stamp: Some(StampMethod::Comment),
        },
    )
    .unwrap();
    assert_eq!(drift, DriftStatus::Clean);

    let new = stamp_meta(None, "hello\n");
    let out = rendered(path.clone(), new.clone(), drift, "hello\n");

    let backend = MaterializeBackend;
    let mut session = backend.prepare(repo_root, &plan).unwrap();
    let report = backend.apply(&mut session, &[out]).unwrap();

    assert_eq!(report.conflicts.len(), 0);
    let after = fs::read_to_string(repo_root.join("out.txt")).unwrap();
    assert!(!after.contains("\"profile\":\"old\""));
}

#[test]
fn refuse_overwrite_unmanaged_with_if_generated() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();

    let path = repo_path_for_existing(repo_root, "out.txt");
    write_file(&repo_root.join("out.txt"), "manual\n");

    let plan = OutputPlan {
        agent_id: "a".to_string(),
        backend: agents_core::model::manifest::BackendKind::Materialize,
        outputs: vec![planned_output(path.clone(), WriteMode::IfGenerated, false)],
    };

    let meta = stamp_meta(None, "hello\n");
    let out = rendered(path.clone(), meta, DriftStatus::Unmanaged, "hello\n");

    let backend = MaterializeBackend;
    let mut session = backend.prepare(repo_root, &plan).unwrap();
    let report = backend.apply(&mut session, &[out]).unwrap();

    assert_eq!(report.written.len(), 0);
    assert_eq!(report.conflicts, vec![path]);
    assert!(report
        .conflict_details
        .iter()
        .any(|c| c.reason == ConflictReason::Unmanaged));
}

#[test]
fn gitignore_update_is_stable_and_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path();

    let path = repo_path_for_existing(repo_root, "out.txt");
    fs::remove_file(repo_root.join("out.txt")).unwrap();

    write_file(
        &repo_root.join(".gitignore"),
        "# user\n\n# BEGIN agents (generated)\nold.txt\n# END agents\n",
    );

    let plan = OutputPlan {
        agent_id: "a".to_string(),
        backend: agents_core::model::manifest::BackendKind::Materialize,
        outputs: vec![planned_output(path.clone(), WriteMode::Always, true)],
    };

    let meta = stamp_meta(None, "hello\n");
    let out = rendered(path.clone(), meta, DriftStatus::Missing, "hello\n");

    let backend = MaterializeBackend;
    let mut session = backend.prepare(repo_root, &plan).unwrap();
    let _ = backend.apply(&mut session, &[out.clone()]).unwrap();
    let first = fs::read_to_string(repo_root.join(".gitignore")).unwrap();

    let mut session2 = backend.prepare(repo_root, &plan).unwrap();
    let _ = backend.apply(&mut session2, &[out]).unwrap();
    let second = fs::read_to_string(repo_root.join(".gitignore")).unwrap();

    assert_eq!(first, second);
    assert!(second.contains("# BEGIN agents (generated)"));
    assert!(second.contains("out.txt"));
}
