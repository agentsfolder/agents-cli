use std::fs;

use agents_core::driftx::{diff_plan, DiffKind};
use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::outputs::plan_outputs;
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::stamps::{apply_stamp, compute_sha256_hex};

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn base_repo(repo: &std::path::Path) {
    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [a] }\n",
    );

    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");

    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );

    write_file(
        &repo.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
}

fn load_and_resolve(
    repo: &std::path::Path,
) -> (
    agents_core::loadag::RepoConfig,
    agents_core::resolv::EffectiveConfig,
) {
    let (cfg, _report) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();

    let resolver = Resolver::new(cfg.clone());
    let mut req = ResolutionRequest::default();
    req.repo_root = repo.to_path_buf();
    let eff = resolver.resolve(&req).unwrap();

    (cfg, eff)
}

fn stamped_comment_for(content_without_stamp: &str) -> String {
    let meta = agents_core::stamps::StampMeta {
        generator: "agents".to_string(),
        adapter_agent_id: "a".to_string(),
        manifest_spec_version: "0.1".to_string(),
        mode: "default".to_string(),
        policy: "safe".to_string(),
        backend: agents_core::model::manifest::BackendKind::VfsContainer,
        profile: None,
        content_sha256: compute_sha256_hex(content_without_stamp),
    };

    apply_stamp(
        content_without_stamp,
        &meta,
        agents_core::model::StampMethod::Comment,
    )
    .unwrap()
}

#[test]
fn diff_noop_when_planned_equals_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    // Create placeholder output file before planning.
    write_file(&repo.join("out.md"), "");

    let (cfg, eff) = load_and_resolve(repo);
    let plan_res = plan_outputs(repo, cfg.clone(), &eff, "a").unwrap();
    let plan = plan_res.plan;

    // Write an existing file that matches planned (including valid stamp).
    write_file(&repo.join("out.md"), &stamped_comment_for("hello\n"));

    let report = diff_plan(repo, &plan).unwrap();
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].kind, DiffKind::Noop);
}

#[test]
fn diff_update_and_diff_when_existing_differs() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    // Create placeholder output file before planning.
    write_file(&repo.join("out.md"), "");

    let (cfg, eff) = load_and_resolve(repo);
    let plan_res = plan_outputs(repo, cfg.clone(), &eff, "a").unwrap();
    let plan = plan_res.plan;

    // Write a stamped existing file with different content.
    write_file(&repo.join("out.md"), &stamped_comment_for("bye\n"));

    let report = diff_plan(repo, &plan).unwrap();
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].kind, DiffKind::Drifted);
    assert!(report.entries[0]
        .unified_diff
        .as_deref()
        .unwrap_or("")
        .contains("-bye"));
    assert!(report.entries[0]
        .unified_diff
        .as_deref()
        .unwrap_or("")
        .contains("+hello"));
}

#[test]
fn diff_unmanaged_exists_when_file_has_no_stamp() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        "agentId: a\nversion: '0.1'\nbackendDefaults: { preferred: vfs_container, fallback: materialize }\noutputs:\n  - path: out.md\n    format: md\n    renderer: { type: template, template: t.hbs }\n    driftDetection: { method: sha256, stamp: comment }\n",
    );
    write_file(&repo.join(".agents/adapters/a/templates/t.hbs"), "hello\n");

    // Workaround: outputs planner currently canonicalizes output paths.
    // Create placeholder output file before planning.
    write_file(&repo.join("out.md"), "");

    let (cfg, eff) = load_and_resolve(repo);
    let plan_res = plan_outputs(repo, cfg.clone(), &eff, "a").unwrap();
    let plan = plan_res.plan;

    write_file(&repo.join("out.md"), "bye\n");

    let report = diff_plan(repo, &plan).unwrap();
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].kind, DiffKind::UnmanagedExists);
}
