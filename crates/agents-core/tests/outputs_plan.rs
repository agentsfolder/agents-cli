use std::fs;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::model::BackendKind;
use agents_core::outputs::plan_outputs;
use agents_core::resolv::{ResolutionRequest, Resolver};

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

    write_file(&repo.join(".agents/profiles/dev.yaml"), "{}\n");
}

fn load_and_resolve(
    repo: &std::path::Path,
    override_backend: Option<BackendKind>,
    override_profile: Option<&str>,
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
    req.override_backend = override_backend;
    req.override_profile = override_profile.map(|s| s.to_string());
    let eff = resolver.resolve(&req).unwrap();

    (cfg, eff)
}

fn plan_paths(plan: &agents_core::outputs::OutputPlan) -> Vec<String> {
    plan.outputs.iter().map(|o| o.path.as_str().to_string()).collect()
}

#[test]
fn condition_filtering_respects_backend_in_and_profile_in() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        r#"agentId: a
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: always.md
    renderer: { type: template, template: always.hbs }
  - path: vfs.md
    condition: { backendIn: [vfs_container] }
    renderer: { type: template, template: vfs.hbs }
  - path: mat.md
    condition: { backendIn: [materialize] }
    renderer: { type: template, template: mat.hbs }
  - path: dev.md
    condition: { profileIn: [dev] }
    renderer: { type: template, template: dev.hbs }
"#,
    );

    write_file(&repo.join(".agents/adapters/a/templates/always.hbs"), "always\n");
    write_file(&repo.join(".agents/adapters/a/templates/vfs.hbs"), "vfs\n");
    write_file(&repo.join(".agents/adapters/a/templates/mat.hbs"), "mat\n");
    write_file(&repo.join(".agents/adapters/a/templates/dev.hbs"), "dev\n");

    // vfs_container + no profile
    let (cfg, eff) = load_and_resolve(repo, None, None);
    let plan_res = plan_outputs(repo, cfg.clone(), &eff, "a").unwrap();
    assert_eq!(plan_paths(&plan_res.plan), vec!["always.md", "vfs.md"]);

    // materialize + dev profile
    let (_cfg, eff) = load_and_resolve(repo, Some(BackendKind::Materialize), Some("dev"));
    let plan_res = plan_outputs(repo, cfg, &eff, "a").unwrap();
    assert_eq!(
        plan_paths(&plan_res.plan),
        vec!["always.md", "dev.md", "mat.md"]
    );
}

#[test]
fn physical_path_collision_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        r#"agentId: a
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: same.md
    renderer: { type: template, template: a.hbs }
  - path: same.md
    renderer: { type: template, template: b.hbs }
"#,
    );
    write_file(&repo.join(".agents/adapters/a/templates/a.hbs"), "a\n");
    write_file(&repo.join(".agents/adapters/a/templates/b.hbs"), "b\n");

    let (cfg, eff) = load_and_resolve(repo, None, None);
    let err = plan_outputs(repo, cfg, &eff, "a").unwrap_err();
    match err {
        agents_core::outputs::PlanError::PathCollision { path } => {
            assert_eq!(path, "same.md");
        }
        other => panic!("expected PathCollision, got: {other:?}"),
    }
}

#[test]
fn shared_owner_surface_enforces_manifest_owner() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    // Override sharedSurfacesOwner to core; adapter `a` must not be allowed to emit shared_owner.
    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
defaults: { mode: default, policy: safe, sharedSurfacesOwner: core }\n\
enabled: { modes: [default], policies: [safe], skills: [], adapters: [a] }\n",
    );

    write_file(
        &repo.join(".agents/adapters/a/adapter.yaml"),
        r#"agentId: a
version: '0.1'
backendDefaults: { preferred: vfs_container, fallback: materialize }
outputs:
  - path: AGENTS.md
    surface: shared:AGENTS.md
    collision: shared_owner
    renderer: { type: template, template: agents.hbs }
"#,
    );
    write_file(&repo.join(".agents/adapters/a/templates/agents.hbs"), "x\n");

    let (cfg, eff) = load_and_resolve(repo, None, None);
    let err = plan_outputs(repo, cfg, &eff, "a").unwrap_err();
    match err {
        agents_core::outputs::PlanError::SurfaceCollision { surface } => {
            assert_eq!(surface, "shared:AGENTS.md");
        }
        other => panic!("expected SurfaceCollision, got: {other:?}"),
    }
}
