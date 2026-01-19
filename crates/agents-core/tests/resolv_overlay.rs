use std::fs;
use std::path::Path;

use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::resolv::{ResolutionRequest, Resolver};

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn user_overlay_is_lowest_precedence() {
    let repo_tmp = tempfile::tempdir().unwrap();
    let repo = repo_tmp.path();

    // Repo config chooses default mode.
    write_file(
        &repo.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: default, policy: safe }\n\
         enabled: { modes: [default], policies: [safe], skills: [], adapters: [] }\n",
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

    // Overlay tries to set a different default mode.
    let overlay_tmp = tempfile::tempdir().unwrap();
    let overlay_root = overlay_tmp.path().to_path_buf();

    write_file(
        &overlay_root.join(".agents/manifest.yaml"),
        "specVersion: '0.1'\n\
         defaults: { mode: overlay, policy: safe }\n\
         enabled: { modes: [overlay], policies: [safe], skills: [], adapters: [] }\n",
    );
    write_file(&overlay_root.join(".agents/prompts/base.md"), "base\n");
    write_file(
        &overlay_root.join(".agents/prompts/project.md"),
        "project\n",
    );
    write_file(
        &overlay_root.join(".agents/modes/overlay.md"),
        "---\nid: overlay\n---\n\n",
    );
    write_file(
        &overlay_root.join(".agents/policies/safe.yaml"),
        "id: safe\ndescription: safe\ncapabilities: {}\npaths: {}\nconfirmations: {}\n",
    );

    let (cfg, _r) = load_repo_config(
        repo,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .unwrap();

    let resolver = Resolver::new(cfg);

    let mut req = ResolutionRequest::default();
    req.repo_root = repo.to_path_buf();
    req.enable_user_overlay = true;
    req.user_overlay_root = Some(overlay_root);

    let eff = resolver.resolve(&req).unwrap();

    // Repo default should win over overlay.
    assert_eq!(eff.mode_id, "default");
}
