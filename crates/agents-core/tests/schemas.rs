use std::path::Path;

#[test]
fn valid_fixture_passes_schema_validation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schemas/valid");

    let (cfg, _report) = agents_core::loadag::load_repo_config(
        &root,
        &agents_core::loadag::LoaderOptions {
            require_schemas_dir: true,
        },
    )
    .unwrap();

    agents_core::schemas::validate_repo_config(&root, &cfg).unwrap();
}

#[test]
fn invalid_fixture_fails_with_pointer_and_schema() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schemas/invalid");

    let (cfg, _report) = agents_core::loadag::load_repo_config(
        &root,
        &agents_core::loadag::LoaderOptions {
            require_schemas_dir: true,
        },
    )
    .unwrap();

    let err = agents_core::schemas::validate_repo_config(&root, &cfg).unwrap_err();

    assert!(err.path.to_string_lossy().contains(".agents/policies"));
    assert_eq!(err.schema, "policy.schema.json");
    assert!(!err.pointer.is_empty() || !err.message.is_empty());
}
