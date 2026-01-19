use std::fs;

use agents_core::loadag::{load_repo_config, LoadError, LoaderOptions};

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn missing_manifest_is_not_initialized() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    fs::create_dir_all(repo.join(".agents")).unwrap();

    let opts = LoaderOptions::default();
    let err = load_repo_config(repo, &opts).unwrap_err();

    match err {
        LoadError::NotInitialized { .. } => {}
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn duplicate_policy_ids_error() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\ndefaults: { mode: default, policy: safe }\nenabled: { modes: [default], policies: [safe], skills: [], adapters: [] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");

    // required by integrity check
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );

    write_file(
        &repo.join(".agents/policies/a.yaml"),
        "id: safe\ndescription: a\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
    write_file(
        &repo.join(".agents/policies/b.yaml"),
        "id: safe\ndescription: b\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    let opts = LoaderOptions::default();
    let err = load_repo_config(repo, &opts).unwrap_err();
    match err {
        LoadError::DuplicateId { kind, id } => {
            assert_eq!(kind, "policies");
            assert_eq!(id, "safe");
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn deterministic_ordering_of_policies_map() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\ndefaults: { mode: default, policy: p1 }\nenabled: { modes: [default], policies: [p1, p2], skills: [], adapters: [] }\n",
    );
    write_file(&repo.join(".agents/prompts/base.md"), "base\n");
    write_file(&repo.join(".agents/prompts/project.md"), "project\n");
    write_file(
        &repo.join(".agents/modes/default.md"),
        "---\nid: default\n---\n\n",
    );

    write_file(
        &repo.join(".agents/policies/z.yaml"),
        "id: p2\ndescription: p2\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );
    write_file(
        &repo.join(".agents/policies/a.yaml"),
        "id: p1\ndescription: p1\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    let opts = LoaderOptions::default();
    let (cfg, _report) = load_repo_config(repo, &opts).unwrap();
    let keys: Vec<_> = cfg.policies.keys().cloned().collect();
    assert_eq!(keys, vec!["p1".to_string(), "p2".to_string()]);
}
