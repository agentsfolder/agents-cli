use agents_core::model;

#[test]
fn deserialize_manifest_minimal() {
    let y = r#"
specVersion: "0.1"
defaults:
  mode: default
  policy: safe
enabled:
  modes: [default]
  policies: [safe]
  skills: []
  adapters: []
"#;

    let m: model::Manifest = serde_yaml::from_str(y).unwrap();
    assert_eq!(m.spec_version, "0.1");
    assert_eq!(m.defaults.mode, "default");
}

#[test]
fn deserialize_policy_minimal() {
    let y = r#"
id: safe
description: safe defaults
capabilities: {}
paths: {}
confirmations: {}
"#;

    let p: model::Policy = serde_yaml::from_str(y).unwrap();
    assert_eq!(p.id, "safe");
}

#[test]
fn deserialize_skill_minimal() {
    let y = r#"
id: example
version: "0.0.1"
title: Example
description: Example skill
activation: instruction_only
interface:
  type: cli
contract:
  inputs: {}
  outputs: {}
requirements:
  capabilities:
    filesystem: none
    exec: restricted
    network: none
"#;

    let s: model::Skill = serde_yaml::from_str(y).unwrap();
    assert_eq!(s.id, "example");
}

#[test]
fn frontmatter_parse_none() {
    let text = "hello\nworld\n";
    let (fm, body) = model::parse_frontmatter_markdown(text).unwrap();
    assert!(fm.is_none());
    assert_eq!(body, "hello\nworld\n");
}

#[test]
fn frontmatter_parse_valid() {
    let text = "---\nid: default\ntitle: Default\n---\nBody\n";
    let (fm, body) = model::parse_frontmatter_markdown(text).unwrap();
    let fm = fm.expect("frontmatter");
    assert_eq!(fm.id.as_deref(), Some("default"));
    assert_eq!(fm.title.as_deref(), Some("Default"));
    assert_eq!(body, "Body\n");
}

#[test]
fn frontmatter_parse_malformed_is_error() {
    let text = "---\nid: [unclosed\n---\nBody\n";
    let err = model::parse_frontmatter_markdown(text).unwrap_err();
    // Don't assert exact message; just ensure it is an error.
    let _ = err;
}

#[test]
fn unknown_fields_are_rejected() {
    let y = r#"
specVersion: "0.1"
defaults:
  mode: default
  policy: safe
enabled:
  modes: [default]
  policies: [safe]
  skills: []
  adapters: []
unknownKey: 123
"#;

    let err = serde_yaml::from_str::<model::Manifest>(y).unwrap_err();
    let _ = err;
}
