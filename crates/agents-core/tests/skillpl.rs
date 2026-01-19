use std::fs;
use std::path::Path;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::skillpl::{SkillPlanError, SkillPlanner};

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn base_repo(tmp: &Path) {
    write_file(
        &tmp.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [s1], adapters: [] }\n",
    );

    write_file(&tmp.join(".agents/prompts/base.md"), "base\n");
    write_file(&tmp.join(".agents/prompts/project.md"), "project\n");

    write_file(
        &tmp.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );

    write_file(
        &tmp.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    write_file(
        &tmp.join(".agents/skills/s1/skill.yaml"),
        "id: s1\nversion: '0.0.1'\ntitle: S1\ndescription: test\nactivation: instruction_only\ninterface: { type: cli }\ncontract: { inputs: {}, outputs: {} }\nrequirements: { capabilities: { filesystem: none, exec: none, network: none } }\n",
    );
}

#[test]
fn enable_disable_precedence_is_deterministic() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    // scope disables s1
    write_file(
        &repo.join(".agents/scopes/a.yaml"),
        "id: a\napplyTo: ['**']\noverrides: { disableSkills: [s1] }\n",
    );

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
    req.target_path = Some("x".to_string());

    let eff = resolver.resolve(&req).unwrap();

    let planner = SkillPlanner::new(cfg);
    let skills = planner.plan(&eff, None).unwrap();

    assert!(skills.enabled.is_empty());
}

#[test]
fn incompatible_backend_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    // Add compatibility for s1 requiring materialize.
    write_file(
        &repo.join(".agents/skills/s1/skill.yaml"),
        "id: s1\nversion: '0.0.1'\ntitle: S1\ndescription: test\nactivation: instruction_only\ninterface: { type: cli }\ncontract: { inputs: {}, outputs: {} }\nrequirements: { capabilities: { filesystem: none, exec: none, network: none } }\ncompatibility: { backends: [materialize] }\n",
    );

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

    let planner = SkillPlanner::new(cfg);
    let err = planner.plan(&eff, None).unwrap_err();

    match err {
        SkillPlanError::IncompatibleBackend { .. } => {}
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn ordering_is_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    // manifest enables skills s2 then s1 (out of order)
    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [s2, s1], adapters: [] }\n",
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

    write_file(
        &repo.join(".agents/skills/s2/skill.yaml"),
        "id: s2\nversion: '0.0.1'\ntitle: S2\ndescription: test\nactivation: instruction_only\ninterface: { type: cli }\ncontract: { inputs: {}, outputs: {} }\nrequirements: { capabilities: { filesystem: none, exec: none, network: none } }\n",
    );
    write_file(
        &repo.join(".agents/skills/s1/skill.yaml"),
        "id: s1\nversion: '0.0.1'\ntitle: S1\ndescription: test\nactivation: instruction_only\ninterface: { type: cli }\ncontract: { inputs: {}, outputs: {} }\nrequirements: { capabilities: { filesystem: none, exec: none, network: none } }\n",
    );

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

    let planner = SkillPlanner::new(cfg);
    let skills = planner.plan(&eff, None).unwrap();

    let ids: Vec<_> = skills.enabled.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids, vec!["s1".to_string(), "s2".to_string()]);
}
