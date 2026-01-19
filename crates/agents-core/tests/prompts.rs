use std::fs;
use std::path::Path;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::model::Policy;
use agents_core::prompts::PromptComposer;
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
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [] }\n",
    );

    write_file(&tmp.join(".agents/prompts/base.md"), "Base\n");
    write_file(&tmp.join(".agents/prompts/project.md"), "Project\n");

    write_file(
        &tmp.join(".agents/modes/default.md"),
        "---\nid: default\nincludeSnippets: [b, a]\n---\n\n",
    );

    write_file(
        &tmp.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: { redact: ['**/secrets/**'] }\nconfirmations: {}\n",
    );

    write_file(&tmp.join(".agents/prompts/snippets/a.md"), "Snippet A\n");
    write_file(&tmp.join(".agents/prompts/snippets/b.md"), "Snippet B\n");
}

#[test]
fn snippet_selection_is_sorted_and_composition_is_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

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

    let policy: Policy = cfg.policies.get(&eff.policy_id).unwrap().clone();

    let composer = PromptComposer::new(repo, cfg);
    let (prompts, sources) = composer.compose(&eff, &policy).unwrap();

    let snippet_ids: Vec<_> = prompts.snippets.iter().map(|s| s.id.clone()).collect();
    assert_eq!(snippet_ids, vec!["a".to_string(), "b".to_string()]);

    // exactly one blank line between sections
    let expected = "Base\n\nProject\n\nSnippet A\n\nSnippet B\n";
    assert_eq!(prompts.composed_md, expected);

    // stable sources order
    let kinds: Vec<_> = sources.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, vec!["base", "project", "snippet", "snippet"]);
}

#[test]
fn redaction_glob_matching_works() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    base_repo(repo);

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
    let policy: Policy = cfg.policies.get(&eff.policy_id).unwrap().clone();

    let composer = PromptComposer::new(repo, cfg);
    let (_prompts, _sources) = composer.compose(&eff, &policy).unwrap();

    let redactor = agents_core::prompts::Redactor::from_policy(&policy).unwrap();
    assert!(redactor.is_redacted("foo/secrets/bar.txt"));
    assert!(!redactor.is_redacted("foo/public/bar.txt"));
}
