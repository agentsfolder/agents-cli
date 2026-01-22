use std::fs;
use std::path::Path;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::model::BackendKind;
use agents_core::resolv::{ResolutionRequest, Resolver};

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
         enabled: { modes: [default, refactor], policies: [safe], skills: [], adapters: [] }\n",
    );
    write_file(&tmp.join(".agents/prompts/base.md"), "base\n");
    write_file(&tmp.join(".agents/prompts/project.md"), "project\n");

    write_file(
        &tmp.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );
    write_file(
        &tmp.join(".agents/modes/refactor.md"),
        "---\nid: refactor\nincludeSnippets: [s1]\n---\n\n",
    );

    write_file(&tmp.join(".agents/prompts/snippets/s1.md"), "Snippet 1\n");

    write_file(
        &tmp.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
}

#[test]
fn cli_override_beats_scope() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/scopes/a.yaml"),
        "id: a\napplyTo: ['apps/**']\noverrides: { mode: default, includeSnippets: [s2] }\n",
    );
    write_file(&repo.join(".agents/prompts/snippets/s2.md"), "Snippet 2\n");

    let (cfg, _r) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();
    let resolver = Resolver::new(cfg);

    let req = ResolutionRequest {
        repo_root: repo.to_path_buf(),
        target_path: Some("apps/web".to_string()),
        override_mode: Some("refactor".to_string()),
        ..Default::default()
    };

    let eff = resolver.resolve(&req).unwrap();
    assert_eq!(eff.mode_id, "refactor");
    assert!(eff.snippet_ids_included.contains(&"s1".to_string()));
}

#[test]
fn scope_beats_repo_default() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/scopes/a.yaml"),
        "id: a\napplyTo: ['apps/**']\noverrides: { mode: refactor }\n",
    );

    let (cfg, _r) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();
    let resolver = Resolver::new(cfg);

    let req = ResolutionRequest {
        repo_root: repo.to_path_buf(),
        target_path: Some("apps/web".to_string()),
        ..Default::default()
    };

    let eff = resolver.resolve(&req).unwrap();
    assert_eq!(eff.mode_id, "refactor");
}

#[test]
fn state_overrides_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/state/state.yaml"),
        "mode: refactor\nbackend: materialize\n",
    );

    let (cfg, _r) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();
    let resolver = Resolver::new(cfg);

    let req = ResolutionRequest {
        repo_root: repo.to_path_buf(),
        ..Default::default()
    };

    let eff = resolver.resolve(&req).unwrap();
    assert_eq!(eff.mode_id, "refactor");
    assert_eq!(eff.backend, BackendKind::Materialize);
}

#[test]
fn specificity_and_priority_break_ties() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

    write_file(
        &repo.join(".agents/scopes/a.yaml"),
        "id: a\napplyTo: ['apps/**']\npriority: 0\noverrides: { mode: default }\n",
    );
    write_file(
        &repo.join(".agents/scopes/b.yaml"),
        "id: b\napplyTo: ['apps/web/**']\npriority: 0\noverrides: { mode: refactor }\n",
    );

    let (cfg, _r) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();
    let resolver = Resolver::new(cfg);

    let req = ResolutionRequest {
        repo_root: repo.to_path_buf(),
        target_path: Some("apps/web/x".to_string()),
        ..Default::default()
    };

    let eff = resolver.resolve(&req).unwrap();
    assert_eq!(eff.mode_id, "refactor");
    assert_eq!(eff.scopes_matched[0].id, "b");

    // Priority tie-break
    write_file(
        &repo.join(".agents/scopes/c.yaml"),
        "id: c\napplyTo: ['apps/web/**']\npriority: 10\noverrides: { mode: default }\n",
    );

    let (cfg2, _r2) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();
    let resolver2 = Resolver::new(cfg2);
    let eff2 = resolver2.resolve(&req).unwrap();

    assert_eq!(eff2.scopes_matched[0].id, "c");
}
